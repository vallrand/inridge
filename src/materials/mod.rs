mod shared;
pub use shared::*;

pub mod displacement;

mod unlit;
pub use unlit::*;
mod terrain;
pub use terrain::*;
mod model;
pub use model::*;
mod color_uniform;
pub use color_uniform::*;
mod nebula;
pub use nebula::*;

mod billboard;
pub use billboard::*;
mod indicator;
pub use indicator::*;
mod matter;
pub use matter::*;
mod tentacle;
pub use tentacle::*;
mod trail;
pub use trail::*;
mod explosion;
pub use explosion::*;
mod condition;
pub use condition::*;
mod scanline;
pub use scanline::*;
mod lightning;
pub use lightning::*;
mod barrier;
pub use barrier::*;

use crate::common::animation::{AnimationSet, animate_component};
use crate::common::rendering::ShaderMaterialPlugin;
use crate::extensions::{RenderTexturePassPlugin, PostProcessPlugin, SkyboxPlugin, FullscreenPhasePlugin};
use bevy::prelude::*;
use bevy::sprite::{Mesh2dHandle,Mesh2dUniform,Material2dPlugin};
use bevy::render::{Extract,RenderApp};
use bevy::window::PrimaryWindow;

pub struct MaterialEffectPlugin; impl Plugin for MaterialEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FullscreenPhasePlugin);
        app.add_plugin(SkyboxPlugin::<nebula::SkyboxNebula>::default());

        app.add_plugin(RenderTexturePassPlugin::<displacement::DisplacementSettings>::default());
        app.add_plugin(PostProcessPlugin::<displacement::DisplacementSettings>::default());
        app.add_plugin(ShaderMaterialPlugin::<displacement::DisplacementMaterial>::default());
        
        app.add_system(animate_component::<color_uniform::ColorUniform>.in_set(AnimationSet::Animator));
        app.add_plugin(ShaderMaterialPlugin::<UnlitMaterial>::default());
        app.add_plugin(ShaderMaterialPlugin::<terrain::LayeredInstancedStandardMaterial>::default());
        app.add_plugin(MaterialPlugin::<terrain::TerrainLayeredMaterial>::default());
        app.add_plugin(MaterialPlugin::<model::ModelEffectLayeredMaterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(color_uniform::ColorUniformPlugin);
        // app.add_plugin(ShaderUniformPlugin::<ColorMask>::default());
        // app.add_plugin(ShaderMaterialPlugin::<model::ModelEffectLayeredMaterial>::default());
        app.add_plugin(Material2dPlugin::<indicator::MatterIndicatorMaterial>::default());
        app.add_plugin(Material2dPlugin::<indicator::RadialIndicatorMaterial>::default());
        app.add_plugin(MaterialPlugin::<billboard::BillboardEffectMaterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(MaterialPlugin::<matter::MatterEffectMaterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(MaterialPlugin::<condition::ReconstructEffectMaterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(MaterialPlugin::<scanline::ScanlineEffectMaterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(MaterialPlugin::<trail::ProjectileTrailMaterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(MaterialPlugin::<tentacle::TentacleEffectMaterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(MaterialPlugin::<explosion::ExplosionMatterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(MaterialPlugin::<lightning::LightningEffectMaterial> { prepass_enabled: false, ..Default::default() });
        app.add_plugin(MaterialPlugin::<barrier::BarrierEffectMaterial> { prepass_enabled: false, ..Default::default() });

        app.sub_app_mut(RenderApp)
            .add_system(extract_ui_mesh2d
                .after(bevy::sprite::extract_mesh2d)
                .in_schedule(ExtractSchedule));
    }
}

fn extract_ui_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Extract<Query<(Entity, &ComputedVisibility, &GlobalTransform, &Mesh2dHandle, &Node)>>,
    primary_window: Extract<Query<&Window, With<PrimaryWindow>>>,
){
    let Ok(primary_window) = primary_window.get_single() else { return };
    let logical_size: Vec2 = Vec2::new(primary_window.resolution.width(), primary_window.resolution.height());

    let mut values = Vec::with_capacity(*previous_len);
    for (entity, visibility, transform, handle, node) in &query {
        if !visibility.is_visible() { continue; }

        let mut transform = transform.compute_matrix();
        transform = transform * Mat4::from_scale(node.size().extend(1.0));
        transform.w_axis.x = transform.w_axis.x - (logical_size.x / 2.0);
        transform.w_axis.y = (logical_size.y / 2.0) - transform.w_axis.y;

        values.push((entity, (
            Mesh2dHandle(handle.0.clone_weak()),
            Mesh2dUniform { flags: 0, transform, inverse_transpose_model: transform.inverse().transpose() },
        )));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}