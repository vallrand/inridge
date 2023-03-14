use bevy::prelude::*;
use bevy::scene::SceneInstance;
use std::f32::consts::FRAC_PI_2;
use crate::common::loader::AssetBundle;
use crate::common::animation::ease::Smoothing;
use crate::logic::{MapGrid, GridTileIndex, GroupLink, LandingProbe};
use crate::scene::ModelAssetBundle;
use crate::effects::animation::{UnitAnimation, MovementFormation, MovementVariant};

pub fn animate_unit_landing(
    time: Res<Time>,
    mut commands: Commands,
    mut query_unit: Query<(Entity, &Parent, &GridTileIndex, &MovementFormation, &mut Transform), With<GroupLink>>,
    query_grid: Query<&MapGrid>
){
    for (entity, parent, tile_index, formation, mut transform) in query_unit.iter_mut() {
        let Ok(grid) = query_grid.get(parent.get()) else { continue };
        let target_transform = &grid.tiles[tile_index.0].transform;
        let fraction = Smoothing::Exponential(0.9).calculate(time.delta_seconds());

        transform.translation = Vec3::lerp(transform.translation, target_transform.translation, fraction);
        transform.scale = Vec3::lerp(transform.scale, target_transform.scale, fraction);
        transform.rotation = Quat::slerp(transform.rotation, target_transform.rotation, fraction);
    }
}

pub fn animate_unit_path_traversal(
    time: Res<Time>,
    mut query_unit: Query<(&MovementFormation, &mut Transform, Option<&LandingProbe>), Without<GroupLink>>,
){
    for (formation, mut transform, landing) in query_unit.iter_mut() {
        transform.translation = formation.sample_position();
        if let MovementVariant::Float { height, frequency, .. } = formation.variant {
            let height_offset = 0.2 * height * (time.elapsed_seconds() * frequency + formation.time_offset).sin();
            transform.translation += formation.sample_normal() * height_offset;
        }
        if let Some(next_rotation) = formation.sample_rotation() {
            transform.rotation = next_rotation;
            if landing.is_some() { transform.rotate_local_z(-FRAC_PI_2); }
        }
    }
}

pub fn animate_unit_movement(
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,
    query_unit: Query<&MovementFormation, Without<GroupLink>>,
    query: Query<(&Parent, &Children, &UnitAnimation), With<SceneInstance>>,
    animations: Res<Assets<AnimationClip>>,
    mut query_animation: Query<&mut AnimationPlayer>,
){
    for (parent, children, animation) in query.iter() {
        let UnitAnimation::Movement(label) = animation else { continue };
        let Ok(formation) = query_unit.get(parent.get()) else { continue };
        let elapsed = formation.elapsed;

        let handle = model_bundle.animations.get(label).unwrap();
        let animation = animations.get(handle).unwrap();

        for &entity in children.iter() {
            let Ok(mut player) = query_animation.get_mut(entity) else { continue };
            player.play(handle.clone_weak());
            player.pause();
            player.set_elapsed(animation.duration() * elapsed);
        }
    }
}