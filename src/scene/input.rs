use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::window::PrimaryWindow;

#[derive(Resource)]
pub struct InputMapping {
    zoom_sensitivity: f32,
    pan_sensitivity: f32,
    rotate_sensitivity: f32,
    pan_toggle: MouseButton,
    rotate_toggle: MouseButton,
}

#[derive(Resource, Clone, Default)]
pub struct InputState {
    pub scroll: f32,
    pub delta: Vec2,
    pub pan: Vec2,
    pub pressed: bool,
    pub prev_position: Vec2,
    pub position: Vec2,
}
impl InputState {
    pub fn reset(&mut self){
        self.scroll = 0.0;
        self.delta = Vec2::ZERO;
        self.pan = Vec2::ZERO;
    }
}

pub fn handle_input_system(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_scroll_events: EventReader<MouseWheel>,
    mouse_buttons: Res<Input<MouseButton>>,
    _keys: Res<Input<KeyCode>>,
    options: Res<InputMapping>,
    mut input_state: ResMut<InputState>,
) {
    input_state.bypass_change_detection().reset();
    let window = window_query.get_single().unwrap();
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
    scroll *= options.zoom_sensitivity;
    if scroll != 0.0 { input_state.scroll = scroll; }

    if mouse_buttons.pressed(options.rotate_toggle) {
        input_state.delta = delta * Vec2::splat(options.rotate_sensitivity);

        if let Some(position) = window.cursor_position() {
            input_state.position = position / screen;
        }
    } else if mouse_buttons.pressed(options.pan_toggle) {
        input_state.pan = delta * Vec2::splat(options.pan_sensitivity);
    }

    if mouse_buttons.just_pressed(options.rotate_toggle) {
        input_state.prev_position = input_state.position;
        input_state.pressed = true;
    }
    if input_state.pressed && !mouse_buttons.pressed(options.rotate_toggle) {
        input_state.pressed = false;
    }
}

pub struct InputManagerPlugin;
impl Plugin for InputManagerPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<InputState>()
        .insert_resource(InputMapping {
            rotate_sensitivity: 1.0,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 0.1,
            pan_toggle: MouseButton::Middle,
            rotate_toggle: MouseButton::Left,
        })
        .add_system(handle_input_system.in_base_set(CoreSet::PreUpdate));
    }
}