use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use std::ops::AddAssign;

#[derive(Component, serde::Deserialize, Clone, Debug)]
pub struct UpgradeDistribution {
    pub effect: UpgradeVariant,
    pub range: i32,
    #[serde(default, skip)] pub list: Vec<usize>,
}

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum UpgradeVariant {
    Amplitude(UpgradeAmplitude),
    Frequency(UpgradeFrequency),
    Range(UpgradeRange),
    Immobilize(DegradeImmobilize),
}
impl UpgradeVariant {
    pub fn apply(&self, mut commands: EntityCommands){
        match self {
            UpgradeVariant::Amplitude(component) => { commands.insert_add(component.clone()); },
            UpgradeVariant::Frequency(component) => { commands.insert_add(component.clone()); },
            UpgradeVariant::Range(component) => { commands.insert_add(component.clone()); },
            UpgradeVariant::Immobilize(component) => { commands.insert_add(component.clone()); },
        }
    }
}

#[derive(Component, serde::Deserialize, Deref, DerefMut, Clone, Copy, Default, Debug)]
pub struct UpgradeRange(pub i32);
impl AddAssign<UpgradeRange> for UpgradeRange {
    #[inline] fn add_assign(&mut self, rhs: UpgradeRange) { self.0 += rhs.0; }
}

#[derive(Component, serde::Deserialize, Deref, DerefMut, Clone, Copy, Default, Debug)]
pub struct UpgradeFrequency(pub i32);
impl AddAssign<UpgradeFrequency> for UpgradeFrequency {
    #[inline] fn add_assign(&mut self, rhs: UpgradeFrequency) { self.0 += rhs.0; }
}

#[derive(Component, serde::Deserialize, Deref, DerefMut, Clone, Copy, Default, Debug)]
pub struct UpgradeAmplitude(pub i32);
impl AddAssign<UpgradeAmplitude> for UpgradeAmplitude {
    #[inline] fn add_assign(&mut self, rhs: UpgradeAmplitude) { self.0 += rhs.0; }
}

#[derive(Component, serde::Deserialize, Deref, DerefMut, Clone, Copy, Default, Debug)]
pub struct DegradeImmobilize(pub i32);
impl AddAssign<DegradeImmobilize> for DegradeImmobilize {
    #[inline] fn add_assign(&mut self, rhs: DegradeImmobilize) { self.0 += rhs.0; }
}

use crate::extensions::CommandsExtension;
use crate::common::adjacency::TraversableGraph;
use crate::logic::{MapGrid, GridTileIndex, GroupLink, MatterBinding, UnderConstruction, Suspended};

pub fn expiration_phase(
    mut commands: Commands,
    query: Query<Entity, Or<(
        With<UpgradeAmplitude>, With<UpgradeFrequency>, With<UpgradeRange>,
        With<DegradeImmobilize>,
    )>>,
){
    for entity in query.iter() {
        commands.entity(entity).remove::<(
            UpgradeAmplitude,UpgradeFrequency,UpgradeRange,
            DegradeImmobilize
        )>();
    }
}

pub fn propagation_phase(
    mut commands: Commands,
    query_grid: Query<&MapGrid>,
    mut query_unit: ParamSet<(
        Query<&mut UpgradeDistribution>,
        Query<(
            &Parent, &GridTileIndex, &mut UpgradeDistribution, &MatterBinding
        ), (Without<UnderConstruction>, Without<Suspended>, With<GroupLink>)>
    )>
){
    for mut upgrade in query_unit.p0().iter_mut() {
        upgrade.list.clear();
    }
    for (parent, tile_index, mut upgrade, matter) in query_unit.p1().iter_mut() {
        let MatterBinding::Consumption(consumption) = matter else { continue };
        if !consumption.active() { continue; }
        let Ok(grid) = query_grid.get(parent.get()) else { continue };
        let Some(group_index) = grid.visited.get(&tile_index) else { continue };

        upgrade.list = grid.graph.iter_breadth_first()
            .with_origin(**tile_index)
            .with_limit(upgrade.range as usize)
            .with_filter(|index|grid.visited.get(&index).map_or(false,|i|i == group_index))
            .collect();

        for &i in upgrade.list.iter().skip(1) {
            let Some(entity) = grid.tiles[i].reference else { continue };
            upgrade.effect.apply(commands.entity(entity));
        }
    }
}