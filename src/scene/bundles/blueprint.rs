use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::reflect::TypeUuid;
use bevy::ecs::system::EntityCommands;
use crate::common::loader::{AssetBundleList, ScopedAssetServer};
use crate::extensions::CommandsExtension;
use crate::effects::animation::{UnitAnimation, MovementVariant, MovementFormation};
use crate::logic::{
    Agent, AreaBlueprint, GlobalEconomy,
    BoundingRadius, UnitDirective, UnderConstruction, Integrity, Velocity,
    MatterBinding, UpgradeDistribution, UnitFabrication, MilitaryBinding, StrategySettings,
};

#[derive(serde::Deserialize, Clone, Default)]
pub struct UnitPlacement {
    pub key: String,
    pub area: usize,
    pub tile: usize,
    pub agent: Agent,
}

#[derive(serde::Deserialize, TypeUuid, Clone, Default)]
#[uuid = "e5dcb5ed-95f4-4061-aea2-09dc6253135f"]
pub struct StageBlueprint {
    pub economy: GlobalEconomy,
    pub strategy: StrategySettings,
    pub areas: Vec<AreaBlueprint>,
    pub units: Vec<UnitPlacement>,
}

#[derive(serde::Deserialize, TypeUuid, Clone, Default, Debug)]
#[uuid = "78112566-820b-43bb-872b-e8eb2f736eab"]
pub struct UnitBlueprint {
    pub key: String,
    pub description: String,

    pub predecessor: Option<String>,
    pub model: String,
    pub scale: f32,
    pub radius: BoundingRadius,
    pub animation: Option<UnitAnimation>,
    pub movement: Option<MovementVariant>,
    pub velocity: Velocity,
    pub action: Option<UnitDirective>,

    pub construction: UnderConstruction,
    pub integrity: Integrity,
    pub matter: Option<MatterBinding>,
    pub upgrade: Option<UpgradeDistribution>,
    pub unit: Option<UnitFabrication>,
    pub military: Option<MilitaryBinding>,
}
impl UnitBlueprint {
    pub fn apply(&self, mut commands: EntityCommands, structure: bool){
        commands.insert(self.radius.clone());
        commands.insert_add(self.integrity.clone());
        if self.velocity.0 != 0 { commands.insert(self.velocity.clone()); }
        if structure { commands.insert(self.construction.clone()); }
        if let Some(movement) = self.movement.as_ref() {
            commands.insert(MovementFormation{ variant: movement.clone(), ..Default::default() });
        }
        if let Some(component) = self.matter.as_ref() { commands.insert_add(component.clone()); }
        if let Some(component) = self.upgrade.as_ref() { commands.insert(component.clone()); }
        if let Some(component) = self.unit.as_ref() { commands.insert(component.clone()); }
        if let Some(component) = self.military.as_ref() { commands.insert(component.clone()); }
    }
}

pub struct BlueprintAssetBundle {
    pub intro_stage: Handle<StageBlueprint>,
    pub unit_blueprints: Vec<Handle<UnitBlueprint>>,
    mapping: HashMap<String, usize>,
}
impl BlueprintAssetBundle {
    pub fn find_unit<'a>(&'a self, key: &String) -> &'a Handle<UnitBlueprint> {
        self.mapping.get(key).map(|&i|&self.unit_blueprints[i]).unwrap()
    }
}
impl AssetBundleList for BlueprintAssetBundle {
    fn from_asset_server(asset_server: &ScopedAssetServer) -> Self { Self {
        intro_stage: asset_server.load("data/intro.stage.ron"),
        unit_blueprints: asset_server.load_folder("data/units"),
        mapping: HashMap::new(),
    } }
    fn prepare(&mut self, world: &mut World) {
        let blueprints = world.resource::<Assets<UnitBlueprint>>();
        for (index, handle) in self.unit_blueprints.iter().enumerate() {
            let blueprint = blueprints.get(handle).unwrap();
            self.mapping.insert(blueprint.key.clone(), index);
        }
    }
}