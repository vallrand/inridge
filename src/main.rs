use bevy::{
    prelude::*
};

mod common;
mod systems;
mod input;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(input::InputManagerPlugin)
    .add_plugin(common::rig::ManipulatorTransformPlugin)
    .add_startup_system(setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
    .insert(common::rig::OrbitManipulatorBundle{
        look: common::rig::LookTransform{ target: Vec3::new(0.0, 0.0, 0.0) },
        distance: common::rig::DistanceConstraint{ distance: 5.0, ..default() },
        ..default()
    });
}