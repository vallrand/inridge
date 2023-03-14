use bevy::prelude::*;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use crate::common::loader::AssetBundle;
use super::{UnitBlueprint, BlueprintAssetBundle, StageBlueprint, EnvironmentAssetBundle, ModelAssetBundle};
use crate::logic::{Agent, NetworkGroupList, ConstructionEvent};
use crate::materials::SkyboxNebula;
use crate::interaction::ViewMode;
use crate::interaction::construct_structure;
use super::lighting::setup_lighting;
use super::GlobalState;

pub fn load_stage(
    mut next_state: ResMut<NextState<GlobalState>>,
    mut commands: Commands,
    mut construction_events: EventWriter<ConstructionEvent>,
    blueprints: Res<Assets<UnitBlueprint>>,
    blueprint_bundle: Res<AssetBundle<BlueprintAssetBundle>>,
    environment_bundle: Res<AssetBundle<EnvironmentAssetBundle>>,
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,
    stages: Res<Assets<StageBlueprint>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mode: ResMut<ViewMode>,
    query_camera: Query<Entity, With<Camera3d>>,
){
    next_state.set(GlobalState::Running);

    let next_stage = stages.get(&blueprint_bundle.intro_stage).unwrap();
    setup_lighting(&mut commands, next_stage);
    commands.insert_resource(next_stage.economy.clone());
    commands.insert_resource(next_stage.strategy.clone());
    *mode = ViewMode::Default(Agent::Player);

    let camera_entity = query_camera.get_single().unwrap();
    commands.entity(camera_entity).insert(SkyboxNebula {
        color: Color::rgb(0.3, 0.4, 0.4),
    }).insert(Camera3d {
        clear_color: ClearColorConfig::None,
        ..Default::default()
    });


    for (area_index, area) in next_stage.areas.iter().enumerate() {
        let (mesh, transform, hitbox, mut grid) = area.load();

        let entity = commands.spawn((
            MaterialMeshBundle {
                mesh: meshes.add(mesh),
                material: environment_bundle.terrain_material.clone(),
                transform, ..Default::default()
            },
            // crate::common::raycast::RaycastTarget::default(),
            hitbox, area.clone(),
            NetworkGroupList::default(),
            crate::effects::linker::TileConnectors::default()
        )).id();

        for placement in next_stage.units.iter().filter(|placement| placement.area == area_index) {
            let handle = blueprint_bundle.find_unit(&placement.key);
            construct_structure(
                &mut commands, &mut construction_events, entity, &mut grid, &model_bundle, &blueprints,
                (handle.clone(), placement.agent, placement.tile, 0), true
            );
        }
        commands.entity(entity).insert(grid);
    }
}