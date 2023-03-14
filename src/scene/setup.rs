use bevy::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::bloom::{BloomSettings, BloomPrefilterSettings, BloomCompositeMode};
use crate::common::{rig, raycast};
use crate::common::animation::ease::{Ease, SimpleCurve, Smoothing};
use crate::materials::displacement::DisplacementSettings;
use super::camera::VirtualCamera;

pub fn setup_scene(mut commands: Commands){
    commands.spawn(Camera3dBundle {
        camera: Camera { order: 0, hdr: true, ..Default::default() },
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Default,
            depth_load_op: Default::default(),
        },
        tonemapping: Tonemapping::TonyMcMapface,
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
    .insert(DepthPrepass)
    .insert(DisplacementSettings {
        intensity: 0.5,
        chromatic_aberration: 0.2,
    })
    .insert(BloomSettings {
        intensity: 0.2,
        prefilter_settings: BloomPrefilterSettings {
            threshold: 1.0, threshold_softness: 0.2,
        },
        composite_mode: BloomCompositeMode::Additive,
        ..BloomSettings::NATURAL
    })
    // .insert(bevy::ui::RelativeCursorPosition::default())
    .insert(raycast::RaycastSource::default())
    .insert(VirtualCamera {
        zoom: 1.0,
        zoom_ease: Ease::In(SimpleCurve::Power(1)),
        zoom_smoothing: Smoothing::Exponential(1.0 - 0.016),
        center_smoothing: Smoothing::Exponential(0.9),
        ..Default::default()
    })
    .insert(rig::OrbitManipulatorBundle{
        look: rig::LookTransform{ target: Vec3::new(0.0, 0.0, 0.0) },
        orientation: rig::OrientationTransform::Free(Quat::IDENTITY),
        distance: rig::DistanceConstraint::from(5.0..15.0),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.84,0.96,1.0), brightness: 0.08,
    });
}