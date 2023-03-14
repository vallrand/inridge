use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use std::f32::consts::FRAC_PI_2;
use crate::common::loader::AssetBundle;
use crate::common::animation::ease::{Ease, SimpleCurve};
use crate::common::animation::{AnimationTimeline, Animator, Track, TransformScale};
use crate::materials::ColorUniform;
use crate::scene::EffectAssetBundle;
use crate::logic::{ImpactEffect};

pub fn animate_impact_effect(
    mut commands: Commands,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    query: Query<(&ImpactEffect, &GlobalTransform), Added<ImpactEffect>>
){
    for (effect, transform) in query.iter() {
        let ImpactEffect::Area { radius, .. } = effect else { continue };

        commands.spawn(SpatialBundle::from_transform(
            Transform::from_matrix(transform.compute_matrix())
            .with_scale(Vec3::splat(*radius))
        ))
        .insert(AnimationTimeline::default().with_cleanup(1.0))
        .with_children(|parent|{
            parent.spawn(ParticleEffectBundle {
                effect: ParticleEffect::new(effect_bundle.particle_hit.clone()),
                ..Default::default()
            });

            parent.spawn((
                SpatialBundle::from_transform(Transform::default()
                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
                    .with_translation(Vec3::new(0.0, 0.2, 0.0))),
                effect_bundle.mesh_quad.clone(),
                effect_bundle.material_wave.clone(),
                ColorUniform::from(Color::WHITE),
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::ZERO.into(), 0.0, Ease::Linear),
                    (Vec3::ZERO.into(), 0.1, Ease::Linear),
                    (Vec3::splat(2.0).into(), 0.9, Ease::Out(SimpleCurve::Power(2))),
                ], 0)),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::WHITE.into(), 0.0, Ease::Linear),
                    (Color::WHITE.into(), 0.1, Ease::Linear),
                    (Color::NONE.into(), 0.9, Ease::In(SimpleCurve::Power(1))),
                ], 0)),
            ));
    
            parent.spawn((
                SpatialBundle::from_transform(Transform::default()
                    .with_translation(Vec3::new(0.0, 0.2, 0.0))
                    .with_scale(Vec3::splat(1.0))
                ),
                effect_bundle.mesh_quad.clone(),
                effect_bundle.material_explosion_toxic.clone(),
                bevy::pbr::NotShadowCaster,
                bevy::pbr::NotShadowReceiver,
                ColorUniform::from(Color::NONE),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::NONE.into(), 0.0, Ease::Linear),
                    (Color::WHITE.into(), 2.4, Ease::Linear),
                ], 0))
            ));
    
            parent.spawn((
                SpatialBundle::from_transform(Transform::default().with_scale(Vec3::ZERO)),
                effect_bundle.mesh_sphere.clone(),
                effect_bundle.material_fresnel.clone(),
                bevy::pbr::NotShadowCaster,
                bevy::pbr::NotShadowReceiver,
                ColorUniform::from(Color::WHITE),
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::ZERO.into(), 0.0, Ease::Linear),
                    (Vec3::splat(1.0).into(), 0.5, Ease::Out(SimpleCurve::Power(3))),
                ], 0)),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::rgba(8.0,16.0,4.0,1.0).into(), 0.0, Ease::Linear),
                    (Color::NONE.into(), 0.5, Ease::Out(SimpleCurve::Sine)),
                ], 0)),
            ));
        });
    }
}