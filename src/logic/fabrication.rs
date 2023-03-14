use bevy::prelude::*;
use crate::common::animation::ease::lerp;

#[derive(Component, Clone, Default)]
pub struct FabricationGate {
    pub filter: u8,
    pub elapsed: i32,
    pub released: i32,
    pub last_released: std::time::Duration,
    pub limit: i32,
    pub path: Vec<usize>,
}

#[derive(Component, serde::Deserialize, Clone, Default, Debug)]
pub struct UnitFabrication {
    pub key: String,
    pub batch: usize,
    pub group: u8,
    #[serde(default, skip)] pub required: i32,
    #[serde(default, skip)] pub consumed: i32,
    #[serde(default, skip)] pub prev_consumed: i32,
}

impl UnitFabrication {
    pub const MILITARY: u8 = 0x01;
    pub const CIVILIAN: u8 = 0x02;
    pub fn calculate(&self, fraction: f32) -> f32 {
        lerp(self.prev_consumed as f32, self.consumed as f32, fraction) / self.required as f32
    }
    pub fn total_metric(&self) -> i32 { 0 }
    pub fn is_ready(&self) -> bool { self.consumed >= self.required }
}

use crate::common::loader::AssetBundle;
use crate::scene::{BlueprintAssetBundle, UnitBlueprint, ModelAssetBundle};
use crate::interaction::construct_unit;
use crate::logic::{Agent, MapGrid, Integrity, UnderConstruction, Suspended, MatterBinding, GroupLink, NetworkGroupList, MilitarySupply};
use crate::logic::{FollowingPath, LandingProbe};
use crate::logic::{UpgradeAmplitude, UpgradeFrequency, UpgradeRange};

pub fn fabrication_phase(
    time: Res<Time>,
    mut commands: Commands,
    blueprint_bundle: Res<AssetBundle<BlueprintAssetBundle>>,
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,
    blueprints: Res<Assets<UnitBlueprint>>,
    query_grid: Query<(&MapGrid, &NetworkGroupList)>,
    mut query_unit: ParamSet<(
        Query<&mut UnitFabrication>,
        Query<(
            &Parent, &Agent, &GroupLink, &MatterBinding, &mut UnitFabrication,
            Option<&UpgradeAmplitude>, Option<&UpgradeFrequency>, Option<&UpgradeRange>
        ), (Without<FabricationGate>, Without<UnderConstruction>, Without<Suspended>)>
    )>,
    mut query_gate: Query<&mut FabricationGate, (Without<Suspended>, With<GroupLink>)>
){
    for mut fabrication in query_unit.p0().iter_mut() {
        fabrication.prev_consumed = fabrication.consumed;
    }
    for (
        parent, agent, group, matter, mut fabrication,
        amplitude, frequency, range
    ) in query_unit.p1().iter_mut() {
        let blueprint_handle = blueprint_bundle.find_unit(&fabrication.key);
        let Some(blueprint) = blueprints.get(blueprint_handle) else { continue };

        fabrication.required = blueprint.construction.required;

        let amplitude = amplitude.map_or(0,|upgrade|upgrade.0);
        let frequency = frequency.map_or(0,|upgrade|upgrade.0);
        let range = range.map_or(0,|upgrade|upgrade.0);

        if !fabrication.is_ready() {
            if let MatterBinding::Consumption(consumption) = matter {
                if !consumption.active() { continue; }
            }    
            fabrication.consumed += 1 + frequency;
            continue;
        }

        let Ok((grid, groups)) = query_grid.get(parent.get()) else { continue };
        let Some(group) = group.0.map(|i|&groups[i]) else { continue };

        let Some((&tile_index, &target_entity,_)) = group.list.iter()
        .filter_map(|(index, entity)|query_gate.get(*entity).ok()
            .filter(|gate|gate.released < gate.limit || gate.limit == 0)
            .map(|gate|(index, entity, gate.released)))
        .min_by_key(|row|row.2) else { continue };
        let Ok(mut gate) = query_gate.get_mut(target_entity) else { continue };
        if gate.filter & fabrication.group == 0 { continue; }

        fabrication.consumed = 0;
        gate.released += 1;
        gate.last_released = time.elapsed();
        for _i in 0..(fabrication.batch as i32 * (1 + amplitude)) {
            let entity = construct_unit(
                &mut commands, parent.get(), &grid, &model_bundle, &blueprints,
                (blueprint_handle.clone(), *agent, tile_index)
            );
            commands.entity(entity).insert(FollowingPath::from(gate.path.clone()));
            if fabrication.group == UnitFabrication::CIVILIAN {
                commands.entity(entity).insert(LandingProbe::default());
            } else if fabrication.group == UnitFabrication::MILITARY {
                commands.entity(entity).insert(MilitarySupply {
                    amplitude, frequency, range, snapshot: true
                });
            }
        }
    }
}

pub fn expiration_phase(
    time: Res<Time>,
    mut commands: Commands,
    mut query: ParamSet<(
        Query<(&mut Integrity, &MatterBinding)>,
        Query<(Entity, &FabricationGate)>
    )>
){
    for (mut integrity, matter) in query.p0().iter_mut() {
        let MatterBinding::Collection(collection) = matter else { continue };
        if collection.stored == 0 && collection.recharge == 0 {
            integrity.apply_damage(i32::MAX);
        }
    }
    for (entity, gate) in query.p1().iter() {
        if gate.limit > 0 && gate.released >= gate.limit {
            let elapsed = (time.elapsed() - gate.last_released).as_secs_f32();
            if elapsed > 2.0 {
                commands.entity(entity).remove::<FabricationGate>();
            }
        }
    }
}