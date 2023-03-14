use bevy::prelude::*;
use crate::common::animation::ease::{lerp, Ease, Smoothing};
use crate::common::rig::{OrientationTransform, DistanceConstraint};
use crate::interaction::GridSelection;
use crate::logic::MapGrid;
use super::input::InputState;

#[derive(Component, Clone, Default)]
pub struct VirtualCamera {
    pub zoom: f32,
    pub zoom_ease: Ease,
    pub zoom_smoothing: Smoothing,
    pub center_smoothing: Smoothing,
    pub prev_rotation: Option<Quat>,
}

pub fn update_camera_view(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut query: Query<(&mut VirtualCamera, &mut OrientationTransform, &mut DistanceConstraint), With<Camera>>,
    query_grid: Query<(&MapGrid, &GridSelection, &GlobalTransform)>
){
    let Ok((mut camera, mut orientation, mut distance)) = query.get_single_mut() else { return };
    if input_state.is_changed() {
        camera.zoom = (camera.zoom - input_state.scroll).clamp(0.0, 1.0);

        match orientation.as_mut() {
            OrientationTransform::Free(rotation) => {
                if input_state.pressed && camera.prev_rotation.is_none() {
                    camera.prev_rotation = Some(rotation.clone());
                } else if !input_state.pressed && camera.prev_rotation.is_some() {
                    camera.prev_rotation = None;
                }
                if let Some(prev_rotation) = camera.prev_rotation {

                    let prev = input_state.position * 2.0 - 1.0;
                    let prev = prev.extend((1.0 - prev.length()).max(0.0)).normalize();
                    let next = input_state.prev_position * 2.0 - 1.0;
                    let next = next.extend((1.0 - next.length()).max(0.0)).normalize();
    
                    let rotate = Quat::from_rotation_arc(prev, next);
        
                    *rotation = (prev_rotation * rotate).normalize();
                }
            },
            OrientationTransform::YawPitchRoll { yaw, pitch, .. } => {
                *yaw += -input_state.delta.x;
                *pitch += -input_state.delta.y;
                orientation.apply_constraits();
            }
        }
    }
    
    let next_distance = lerp(distance.range.0, distance.range.1, camera.zoom_ease.calculate(camera.zoom));
    distance.distance = lerp(distance.distance, next_distance, camera.zoom_smoothing.calculate(time.delta_seconds()));

    if camera.prev_rotation.is_some() { return; }
    let Ok((grid, selection, transform)) = query_grid.get_single() else { return };
    let OrientationTransform::Free(rotation) = orientation.as_mut() else { return };
    let prev_rotation = *rotation;

    let grid_center_position = transform.translation();
    let tile_center_position = transform.transform_point(grid.tiles[selection.0].transform.translation);
    let next_normal = (tile_center_position - grid_center_position).normalize();
    let prev_normal = rotation.mul_vec3(Vec3::Z);
    let rotate = Quat::from_rotation_arc(prev_normal, next_normal);
    *rotation = Quat::slerp(prev_rotation, rotate * prev_rotation, camera.center_smoothing.calculate(time.delta_seconds()));
}