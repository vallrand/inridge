use std::ops::AddAssign;
use bevy::{
    prelude::*,
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
};

#[derive(Resource)]
pub struct InputMapping {
    zoom_sensitivity: f32,
    pan_sensitivity: f32,
    rotate_sensitivity: f32,
    pan_toggle: MouseButton,
    rotate_toggle: MouseButton,
}

fn handle_input_system(
    windows: Res<Windows>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_scroll_events: EventReader<MouseWheel>,
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    options: Res<InputMapping>,
    mut query: Query<(&mut crate::common::rig::OrientationTransform, &mut crate::common::rig::DistanceConstraint), With<Camera>>,
) {
    let window = windows.get_primary().unwrap();
    let screen = Vec2::new(window.width(), window.height());

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        delta += event.delta;
    }
    delta *= std::f32::consts::PI / screen;
    let mut scroll = 0.0;
    for event in mouse_scroll_events.iter() {
        scroll += event.y * match event.unit {
            MouseScrollUnit::Line => 1.0,
            MouseScrollUnit::Pixel => 0.1,
        };
    }

    if mouse_buttons.pressed(options.pan_toggle) {
        delta *= Vec2::splat(options.pan_sensitivity);
    } else if mouse_buttons.pressed(options.rotate_toggle) {
        delta *= Vec2::splat(options.rotate_sensitivity);
    } else {
        delta *= 0.0;
    }
    scroll *= options.zoom_sensitivity;

    if let Ok((mut orientation, mut distance)) = query.get_single_mut() {
        orientation.add_assign(-delta);
        distance.add_assign(-scroll);
    }
}

pub struct InputManagerPlugin;
impl Plugin for InputManagerPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(InputMapping {
            rotate_sensitivity: 1.0,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 1.0,
            pan_toggle: MouseButton::Middle,
            rotate_toggle: MouseButton::Left,
        })
        .add_system(handle_input_system);
    }
}