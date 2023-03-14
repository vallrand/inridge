use bevy::prelude::*;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::render::camera::CameraOutputMode;
use bevy::render::render_resource::{LoadOp, BlendState};
use crate::common::loader::AssetBundle;
use crate::scene::InterfaceAssetBundle;

#[derive(Resource)]
pub struct OverlayLayout {
    pub quadrants: [Entity; 4],
    pub inner_radius: f32,
}
impl Default for OverlayLayout {
    fn default() -> Self { Self {
        quadrants: [Entity::PLACEHOLDER; 4],
        inner_radius: 90.0,
    } }
}
impl OverlayLayout {
    pub fn radial_placement(&self, radius: f32, angle: f32, size: f32, quadrant: u8) -> UiRect {
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let origin_x = (quadrant >> 1 & 1) as f32 * 100.0;
        let origin_y = (quadrant & 1) as f32 * 100.0;
        UiRect::new(
            Val::Percent(origin_x + x - size/2.0), Val::Auto, 
            Val::Percent(origin_y + y - size/2.0), Val::Auto
        )
    }
}

pub fn setup_interface_view(
    mut commands: Commands,
    mut layout: ResMut<OverlayLayout>,
    interface_bundle: Res<AssetBundle<InterfaceAssetBundle>>,
){
    commands.spawn(Camera2dBundle {
        camera: Camera { order: 1, hdr: false, output_mode: CameraOutputMode::Write {
            blend_state: Some(BlendState::ALPHA_BLENDING),
            color_attachment_load_op: LoadOp::Load,
        }, ..Default::default() },
        tonemapping: Tonemapping::None,
        camera_2d: Camera2d { clear_color: ClearColorConfig::Custom(Color::NONE) },
        ..Default::default()
    });

    let right = commands.spawn(NodeBundle { style: Style {
        size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
        flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::End,
        ..Default::default()
    }, ..Default::default() }).id();
    let left = commands.spawn(NodeBundle { style: Style {
        size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
        flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Start,
        ..Default::default()
    }, ..Default::default() }).id();

    let quadrant_style = Style {
        size: Size::height(Val::Percent(50.0)), aspect_ratio: Some(1.0),
        min_size: Size::all(Val::Percent(0.0)), max_size: Size::width(Val::Percent(100.0)),
        ..Default::default()
    };

    let left_top = commands.spawn(ImageBundle {
        style: quadrant_style.clone(),
        image: UiImage{ texture: interface_bundle.panel_quadrant.clone(), flip_x: false, flip_y: false },
        background_color: interface_bundle.color_overlay.into(),
        ..Default::default()
    }).id();
    let left_bottom = commands.spawn(ImageBundle {
        style: quadrant_style.clone(),
        image: UiImage{ texture: interface_bundle.panel_quadrant.clone(), flip_x: false, flip_y: true },
        background_color: interface_bundle.color_overlay.into(),
        ..Default::default()
    }).id();
    let right_top = commands.spawn(ImageBundle {
        style: quadrant_style.clone(),
        image: UiImage{ texture: interface_bundle.panel_quadrant.clone(), flip_x: true, flip_y: false },
        background_color: interface_bundle.color_overlay.into(),
        ..Default::default()
    }).id();
    let right_bottom = commands.spawn(ImageBundle {
        style: quadrant_style.clone(),
        image: UiImage{ texture: interface_bundle.panel_quadrant.clone(), flip_x: true, flip_y: true },
        background_color: interface_bundle.color_overlay.into(),
        ..Default::default()
    }).id();

    commands.entity(left).push_children(&[left_top,left_bottom]);
    commands.entity(right).push_children(&[right_top,right_bottom]);

    layout.quadrants = [right_bottom, right_top, left_bottom, left_top];
}