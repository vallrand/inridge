use bevy::prelude::*;
use std::ops::{AddAssign, Range};

///Rotation as three axis angles. <https://en.wikipedia.org/wiki/Aircraft_principal_axes>
#[derive(Component, Clone, Copy)]
pub enum OrientationTransform {
    YawPitchRoll {
        yaw: f32,
        pitch: f32,
        roll: f32,
        pitch_range: (f32, f32),
    },
    Free(Quat)
}
impl OrientationTransform {
    pub fn apply_constraits(&mut self){
        match self {
            OrientationTransform::YawPitchRoll { pitch, pitch_range, .. } => {
                let clamped_pitch = *pitch;
                *pitch = clamped_pitch.clamp(pitch_range.0, pitch_range.1);
            },
            _ => {}
        }
    }
}

impl From<&OrientationTransform> for Quat {
    fn from(value: &OrientationTransform) -> Self {
        match value {
            &OrientationTransform::YawPitchRoll { yaw, pitch, roll, .. } => {
                let mut rotation = Quat::from_axis_angle(Vec3::Y, yaw) *
                Quat::from_axis_angle(Vec3::X, pitch);
                if roll != 0.0 { rotation *= Quat::from_axis_angle(Vec3::Z, roll); }
                rotation
            },
            &OrientationTransform::Free(rotation) => rotation
        }
    }
}
impl Default for OrientationTransform {
    fn default() -> Self { Self::YawPitchRoll {
        yaw: 0.0, pitch: 0.1, roll: 0.0,
        pitch_range: (0.0 + 0.1, std::f32::consts::PI - 0.1)
    } }
}

///Rotate towards the target.
#[derive(Component, Clone, Copy)]
pub struct LookTransform {
    pub target: Vec3
}
impl Default for LookTransform {
    fn default() -> Self { Self { target: Vec3::ZERO } }
}

#[derive(Component, Clone, Copy, Default)]
pub struct DistanceConstraint {
    pub distance: f32,
    pub range: (f32, f32)
}
impl AddAssign<f32> for DistanceConstraint {
    fn add_assign(&mut self, rhs: f32) {
        self.distance += rhs;
        self.distance = self.distance.clamp(self.range.0, self.range.1);
    }
}
impl From<Range<f32>> for DistanceConstraint {
    fn from(value: Range<f32>) -> Self { Self { range: (value.start, value.end), distance: value.start } }
}

fn update_free_transform_system(
    mut query: Query<(&mut Transform, &OrientationTransform),
    (Changed<OrientationTransform>, Without<LookTransform>)>
) {
    for (mut transform, orientation) in query.iter_mut() {
        transform.rotation = orientation.into();
    }
}
fn update_follow_transform_system(
    mut query: Query<(&mut Transform, &LookTransform),
    (Changed<LookTransform>, Without<OrientationTransform>)>
) {
    for (mut transform, lookat) in query.iter_mut() {
        transform.look_at(lookat.target, Vec3::Y);
    }
}
fn update_orbit_transform_system(
    mut query: Query<(&mut Transform, &OrientationTransform, &LookTransform, &DistanceConstraint),
    Or<(Changed<OrientationTransform>, Changed<LookTransform>, Changed<DistanceConstraint>)>>
) {
    for (mut transform, orientation, lookat, distance) in query.iter_mut() {
        let rotation = Quat::from(orientation);
        transform.rotation = rotation;
        transform.translation = (rotation * Vec3::Z) * distance.distance + lookat.target;
    }
}

#[derive(Bundle, Default)]
pub struct OrbitManipulatorBundle {
    pub orientation: OrientationTransform,
    pub look: LookTransform,
    pub distance: DistanceConstraint
}

pub struct ManipulatorTransformPlugin;
impl Plugin for ManipulatorTransformPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system(update_free_transform_system)
        .add_system(update_follow_transform_system)
        .add_system(update_orbit_transform_system);
    }
}
