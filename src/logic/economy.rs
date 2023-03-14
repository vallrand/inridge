use bevy::prelude::*;
use std::ops::AddAssign;
use crate::common::animation::ease::lerp;

#[derive(Resource, serde::Deserialize, Clone, Default)]
pub struct GlobalEconomy {
    pub density: Vec<i32>,
    #[serde(default, skip)] pub priority: u64,
}
impl GlobalEconomy {
    pub fn next_priority(&mut self) -> u64 { self.priority += 1; self.priority }
}

#[derive(Clone, Default)]
pub struct EconomySummary {
    pub matter: i32,
    pub matter_reservation: i32,
    pub matter_consumption: i32,
    pub matter_production: i32,
}

#[derive(Component, serde::Deserialize, Clone, Debug)]
pub enum MatterBinding {
    Production(MatterProduction),
    Consumption(MatterConsumption),
    Collection(MatterStorage),
}

impl AddAssign<MatterBinding> for MatterBinding {
    fn add_assign(&mut self, rhs: MatterBinding) { match (self, rhs) {
        (MatterBinding::Collection(lhs), MatterBinding::Collection(rhs)) => {
            let MatterStorage { stored, prev_stored, .. } = *lhs;
            lhs.clone_from(&rhs);
            lhs.stored += stored;
            lhs.prev_stored = prev_stored;
        },
        (lhs, rhs) => {
            *lhs = rhs;
        }
    } }
}

#[derive(serde::Deserialize, Clone, Default, Debug)]
pub struct MatterProduction {
    pub efficiency: i32,
    #[serde(default, skip)] pub extracted: i32,
}

#[derive(serde::Deserialize, Clone, Default, Debug)]
pub struct MatterConsumption {
    pub quota: i32,
    #[serde(default, skip)] pub calculated: i32,
    #[serde(default, skip)] pub transfered: i32,
}
impl MatterConsumption {
    pub fn active(&self) -> bool { self.calculated >= self.quota && self.transfered >= self.calculated }
}

#[derive(serde::Deserialize, Clone, Default, Debug)]
pub struct MatterStorage {
    pub key: String,
    pub capacity: i32,
    pub stored: i32,
    pub recharge: i32,
    pub discharge: i32,
    #[serde(default, skip)] pub reserved: i32,
    #[serde(default, skip)] pub prev_stored: i32,
}
impl MatterStorage {
    pub fn calculate(&self, fraction: f32) -> f32 {
        lerp(self.prev_stored as f32, self.stored as f32, fraction) / self.capacity as f32
    }
    pub fn tier(&self) -> i32 { self.capacity / 50 }
    pub fn delta(&self) -> i32 { self.stored - self.prev_stored }
}

use crate::logic::{MapGrid, GridTileIndex, GroupLink, NetworkGroupList};
use crate::logic::{Integrity, Suspended, UnderConstruction, UpgradeAmplitude, UpgradeFrequency};

pub fn reset_economy_phase(
    mut query_grid: Query<&mut NetworkGroupList>,
    mut query_unit: Query<(&mut MatterBinding, Option<&Integrity>)>,
){
    for mut groups in query_grid.iter_mut() {
        for group in groups.iter_mut() {
            group.summary.matter = 0;
            group.summary.matter_production = 0;
            group.summary.matter_consumption = 0;
            group.summary.matter_reservation = 0;
        }
    }
    for (mut matter, integrity) in query_unit.iter_mut() {
        match matter.as_mut() {
            MatterBinding::Production(production) => {
                production.extracted = 0;
            },
            MatterBinding::Consumption(consumption) => {
                consumption.calculated = 0;
                consumption.transfered = 0;
            },
            MatterBinding::Collection(storage) => {
                storage.reserved = 0;
                storage.prev_stored = storage.stored;
                if integrity.is_none() { storage.stored = 0; }
            }
        }
    }
}

pub fn production_phase(
    economy: Res<GlobalEconomy>,
    mut query_grid: Query<(&MapGrid, &mut NetworkGroupList)>,
    mut query_unit: Query<(
        &Parent, &GridTileIndex, &GroupLink, &mut MatterBinding,
        Option<&UpgradeFrequency>, Option<&UpgradeAmplitude>,
    ), (Without<UnderConstruction>, Without<Suspended>)>
){
    for (
        parent, tile_index, group,
        mut matter, frequency, amplitude,
    ) in query_unit.iter_mut() {
        let MatterBinding::Production(production) = matter.as_mut() else { continue };
        let Ok((grid, mut groups)) = query_grid.get_mut(parent.get()) else { continue };
        let Some(group) = group.map(|i|&mut groups[i]) else { continue };

        let tile = &grid.tiles[**tile_index];
        let density: i32 = economy.density[tile.variant];
        let frequency = frequency.map_or(0,|upgrade|upgrade.0);
        let amplitude = amplitude.map_or(0,|upgrade|upgrade.0);
        production.extracted = (density + amplitude) * (production.efficiency + frequency);

        group.summary.matter_production += production.extracted;
    }
}

pub fn reservation_phase(
    mut query_grid: Query<&mut NetworkGroupList>,
    mut query_unit: Query<(
        &Parent, &GroupLink, &mut MatterBinding, Option<&UpgradeAmplitude>
    ), (Without<UnderConstruction>, Without<Suspended>)>
){
    for (parent, group, mut matter, amplitude) in query_unit.iter_mut() {
        let MatterBinding::Collection(storage) = matter.as_mut() else { continue };
        let Ok(mut groups) = query_grid.get_mut(parent.get()) else { continue };
        let Some(group) = group.map(|i|&mut groups[i]) else { continue };

        let amplitude = 1 + amplitude.map_or(0,|upgrade|upgrade.0);
        storage.reserved = storage.discharge.min(storage.stored);
        group.summary.matter_reservation += amplitude * storage.reserved;
    }
}

pub fn resource_allocation_phase(
    mut query_grid: Query<&mut NetworkGroupList>,
    mut query_unit: Query<(
        Option<&mut MatterBinding>, Option<&mut UnderConstruction>,
        Option<&UpgradeAmplitude>, Option<&UpgradeFrequency>,
    ), (With<GroupLink>, Without<Suspended>)>
){
    for mut groups in query_grid.iter_mut() {
        for mut group in groups.iter_mut() {
            group.summary.matter = group.summary.matter_production + group.summary.matter_reservation;

            for &(_, entity) in group.list.iter() {
                let Ok((
                    mut matter, construction,
                    amplitude, frequency,
                )) = query_unit.get_mut(entity) else { continue };

                if let Some(mut construction) = construction {
                    let amplitude = 1 + amplitude.map_or(0,|upgrade|upgrade.0);
                    let frequency = 1 + frequency.map_or(0,|upgrade|upgrade.0);

                    let delta = group.summary.matter.min(frequency).max(0);

                    group.summary.matter -= delta;
                    construction.matter_consumed += amplitude * delta;
                    continue;
                }
                if let Some(MatterBinding::Consumption(consumption)) = matter.as_deref_mut() {
                    consumption.calculated = consumption.quota;
                    group.summary.matter_consumption += consumption.calculated;
    
                    consumption.transfered = group.summary.matter.min(consumption.calculated).max(0);
                    group.summary.matter -= consumption.transfered;
                }
            }
        }
    }
}

pub fn collection_phase(
    mut query_grid: Query<&mut NetworkGroupList>,
    mut query_unit: Query<(
        &mut MatterBinding, Option<&UpgradeAmplitude>, Option<&UpgradeFrequency>,
    ), (With<GroupLink>, Without<UnderConstruction>, Without<Suspended>)>
){
    for mut groups in query_grid.iter_mut() {
        for mut group in groups.iter_mut() {
            let mut overflow = (group.summary.matter - group.summary.matter_reservation).max(0);
            let mut ammortization = group.summary.matter_reservation.min(group.summary.matter).max(0);

            for &(_, entity) in group.list.iter() {
                let Ok((
                    mut matter, amplitude, frequency
                )) = query_unit.get_mut(entity) else { continue };
                let MatterBinding::Collection(storage) = matter.as_mut() else { continue };

                let frequency = 1 + frequency.map_or(0,|upgrade|upgrade.0);
                let amplitude = 1 + amplitude.map_or(0,|upgrade|upgrade.0);

                let delta_reserved = storage.reserved.min(ammortization / amplitude);
                ammortization -= delta_reserved * amplitude;
                storage.stored = storage.stored - storage.reserved + delta_reserved;

                let delta_recharge = (storage.recharge * frequency).min(overflow).min(storage.capacity - storage.stored);
                overflow -= delta_recharge;
                storage.stored += delta_recharge;
            }
            group.summary.matter = overflow;
        }
    }
}