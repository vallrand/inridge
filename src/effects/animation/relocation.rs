use std::f32::consts::FRAC_PI_3;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use crate::common::loader::AssetBundle;
use crate::common::animation::ease::{Ease, SimpleCurve};
use crate::scene::ModelAssetBundle;
use crate::logic::{MapGrid, FollowingPath, GridTileIndex};
use crate::effects::animation::{UnitAnimation, MovementFormation};

pub fn animate_unit_relocation(
    fixed_time: Res<FixedTime>,
    model_bundle: Res<AssetBundle<ModelAssetBundle>>,
    mut query_unit: Query<(&Parent, &FollowingPath, &mut Transform), (Without<MovementFormation>, With<GridTileIndex>)>,
    mut query_grid: Query<&mut MapGrid>,
    query: Query<(&Parent, &Children, &UnitAnimation), With<SceneInstance>>,
    animations: Res<Assets<AnimationClip>>,
    mut query_animation: Query<&mut AnimationPlayer>,
){
    let fraction = fixed_time.accumulated().as_secs_f32() / fixed_time.period.as_secs_f32();
    for (parent, children, animation) in query.iter() {
        let UnitAnimation::HexMovement(move_0, move_1, move_2) = animation else { continue };

        let Ok((parent, movement, mut transform)) = query_unit.get_mut(parent.get()) else { continue };
        let Ok(mut grid) = query_grid.get_mut(parent.get()) else { continue };

        let (prev_index, next_index, progress) = movement.calculate_segment(fraction);

        let prev_normal = grid.tiles[prev_index].normal;
        let next_normal = grid.tiles[next_index].normal;
        let prev_rotation = grid.tiles[prev_index].transform.rotation;
        let next_rotation = grid.tiles[next_index].transform.rotation;

        let direction = (next_normal - prev_normal).normalize();
        let prev_direction_local = prev_rotation.inverse().mul_vec3(direction);
        let next_direction_local = next_rotation.inverse().mul_vec3(direction);

        let prev_angle = (prev_direction_local.z.atan2(prev_direction_local.x) / FRAC_PI_3).round();
        let next_angle = (next_direction_local.z.atan2(next_direction_local.x) / FRAC_PI_3).round();
        if prev_angle != next_angle {
            let adjust_rotation = Quat::from_axis_angle(Vec3::Y, (prev_angle - next_angle) * FRAC_PI_3);
            grid.tiles[next_index].transform.rotation = grid.tiles[next_index].transform.rotation * adjust_rotation;
        }

        let tween = Ease::InOut(SimpleCurve::Power(2));
        let progress_tweened = tween.calculate(progress);

        transform.rotation = Quat::slerp(grid.tiles[prev_index].transform.rotation, grid.tiles[next_index].transform.rotation, progress_tweened);
        transform.translation = Vec3::lerp(grid.tiles[prev_index].transform.translation, grid.tiles[next_index].transform.translation, progress_tweened);
        transform.scale = Vec3::lerp(grid.tiles[prev_index].transform.scale, grid.tiles[next_index].transform.scale, progress_tweened);

        let revolution = (prev_angle as i32).rem_euclid(6);
        let (label, speed) = match revolution {
            0 => (move_0, -1),
            1 => (move_1, -1),
            2 => (move_2, -1),
            3 => (move_0, 1),
            4 => (move_1, 1),
            5 => (move_2, 1),
            _ => panic!("out of range {} > 5", revolution)
        };

        let handle = model_bundle.animations.get(label).unwrap();
        let animation = animations.get(handle).unwrap();

        for &entity in children.iter() {
            let Ok(mut player) = query_animation.get_mut(entity) else { continue };
            player.play(handle.clone_weak());
            player.pause();
            if speed > 0 {
                player.set_elapsed(animation.duration() * progress);
            } else {
                player.set_elapsed(animation.duration() * (1.0 - progress));
            }
        }
    }
}

