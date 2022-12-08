use bevy::prelude::*;

///Rotation as three axis angles. <https://en.wikipedia.org/wiki/Aircraft_principal_axes>
#[derive(Component, Clone, Copy)]
pub struct OrientationTransform {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub pitch_range: (f32, f32)
}
impl std::ops::AddAssign<Vec2> for OrientationTransform {
    fn add_assign(&mut self, rhs: Vec2) {
        self.yaw += rhs.x;
		self.pitch += rhs.y;
        self.pitch = self.pitch.clamp(self.pitch_range.0, self.pitch_range.1);
    }
}
impl From<&OrientationTransform> for Quat {
    fn from(val: &OrientationTransform) -> Self {
        Quat::from_axis_angle(Vec3::Y, val.yaw) * Quat::from_axis_angle(Vec3::X, val.pitch)
    }
}
impl Default for OrientationTransform {
    fn default() -> Self { Self {
        yaw: 0.0, pitch: 0.0, roll: 0.0,
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

#[derive(Component, Clone, Copy)]
pub struct DistanceConstraint {
    pub distance: f32,
    pub range: (f32, f32)
}
impl std::ops::AddAssign<f32> for DistanceConstraint {
    fn add_assign(&mut self, rhs: f32) {
        self.distance += rhs;
        self.distance = self.distance.clamp(self.range.0, self.range.1);
    }
}
impl Default for DistanceConstraint {
    fn default() -> Self { Self { distance: 0.0, range: (0.0, 100.0) } }
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
        transform.translation = (rotation * Vec3::Y) * distance.distance + lookat.target;
        transform.look_at(lookat.target, Vec3::Y);
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
