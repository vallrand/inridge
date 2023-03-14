use bevy::prelude::*;
use crate::common::loader::{AssetBundleList, ScopedAssetServer};
use super::texture::TextureSamplerOptions;

pub struct InterfaceAssetBundle {
    pub panel_single: Handle<Image>,
    pub panel_extended: Handle<Image>,
    pub panel_quadrant: Handle<Image>,
    pub arrow: Handle<Image>,
    pub icon_toggle: Handle<Image>,
    pub icon_remove: Handle<Image>,
    pub icon_matter: Handle<Image>,
    pub icon_radius: Handle<Image>,
    pub icon_shield: Handle<Image>,
    pub icon_build: Handle<Image>,
    pub icon_rate: Handle<Image>,
    pub icon_store: Handle<Image>,
    pub icon_action: Handle<Image>,

    pub text_style_secondary: TextStyle,
    pub text_style_primary: TextStyle,

    pub color_overlay: Color,
    pub color_disabled: Color,
    pub color_enabled: Color,
    pub color_active: Color,
    pub color_secondary: Color,

    pub matter_material: Handle<crate::materials::MatterIndicatorMaterial>,
    pub radial_material: Handle<crate::materials::RadialIndicatorMaterial>,
    pub quad_mesh: Handle<Mesh>,
}
impl AssetBundleList for InterfaceAssetBundle {
    fn from_asset_server(asset_server: &ScopedAssetServer) -> Self { Self {
        panel_single: asset_server.load("textures/interface_radial_base.png"),
        panel_extended: asset_server.load("textures/interface_radial_longbase.png"),
        panel_quadrant: asset_server.load("textures/interface_overlay_quadrant.png"),
        arrow: asset_server.load("textures/interface_arrow.png"),
        icon_toggle: asset_server.load("textures/interface_icon_t.png"),
        icon_remove: asset_server.load("textures/interface_icon_x.png"),
        icon_matter: asset_server.load("textures/interface_icon_m.png"),
        icon_radius: asset_server.load("textures/interface_icon_o.png"),
        icon_shield: asset_server.load("textures/interface_icon_h.png"),
        icon_build: asset_server.load("textures/interface_icon_c.png"),
        icon_rate: asset_server.load("textures/interface_icon_i.png"),
        icon_store: asset_server.load("textures/interface_icon_k.png"),
        icon_action: asset_server.load("textures/interface_icon_f.png"),

        text_style_primary: TextStyle {
            font: asset_server.load("fonts/Exo-Black.ttf"), font_size: 40.0,
            color: Color::rgb(0.8,1.0,1.0),
        },
        text_style_secondary: TextStyle {
            font: asset_server.load("fonts/Exo-Black.ttf"), font_size: 30.0,
            color: Color::rgb(0.6,1.0,0.8),
        },

        color_overlay: Color::rgba(0.2, 1.2, 0.8, 0.75),
        color_disabled: Color::rgba(0.4,0.4,0.4, 0.8),
        color_enabled: Color::rgba(0.4,1.0,0.8,1.0),
        color_active: Color::rgba(0.8,1.0,0.4,1.0),
        color_secondary: Color::rgba(0.2,0.8,0.6,1.0),

        matter_material: Default::default(),
        radial_material: Default::default(),
        quad_mesh: Default::default(),
    } }
    fn prepare(&mut self, world: &mut World) {
        let mut images = world.resource_mut::<Assets<Image>>();
        TextureSamplerOptions::DEFAULT.apply(images.get_mut(&self.arrow).unwrap());

        self.matter_material = world.resource_mut::<Assets<crate::materials::MatterIndicatorMaterial>>()
        .add(crate::materials::MatterIndicatorMaterial {
            texture: self.panel_extended.clone(),
            color: Color::rgb(0.2,1.0,0.6), alpha_cutoff: 0.1, fraction_width: 0.02,
            fraction: 0.5,
        });
        self.radial_material = world.resource_mut::<Assets<crate::materials::RadialIndicatorMaterial>>()
        .add(crate::materials::RadialIndicatorMaterial {
            padding: Vec2::new(0.02, 0.04), grid_resolution: Vec2::new(24.0,8.0),
            inner_color: Color::rgb(0.5,0.1,0.3),
            outer_color: Color::rgb(0.6,1.0,0.2),
            radius: Vec2::new(0.52,0.64), sectors: 4, fraction: 0.0
        });

        self.quad_mesh = world.resource_mut::<Assets<Mesh>>()
        .add(shape::Quad::new(Vec2::ONE).into());
    }
}