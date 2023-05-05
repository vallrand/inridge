use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_kira_audio::prelude::*;
use std::f32::consts::FRAC_PI_2;
use crate::common::loader::AssetBundle;
use crate::common::animation::ease::{Ease, SimpleCurve, lerp};
use crate::common::animation::{AnimationTimeline, Animator, Track, TransformScale};
use crate::materials::ColorUniform;
use crate::logic::{CombatEvent, BoundingRadius};
use crate::scene::{EffectAssetBundle, AudioAssetBundle};

pub fn animate_hit_effect(
    time: Res<Time>,
    mut commands: Commands,
    mut events: EventReader<CombatEvent>,
    audio: Res<Audio>,
    audio_bundle: Res<AssetBundle<AudioAssetBundle>>,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    query_unit: Query<&GlobalTransform, With<BoundingRadius>>,
){
    for event in events.iter() {

        let CombatEvent::ProjectileHit(_projectile, target) = event else { continue };
        let Ok(transform) = query_unit.get(*target) else { continue };

        commands.spawn((
            SpatialBundle::from_transform(Transform::from_matrix(transform.compute_matrix())),
            AnimationTimeline::default().with_cleanup(1.0),
        ))
        .insert(AudioEmitter{instances:vec![
            audio.play(audio_bundle.hit_impact.clone())
            .with_playback_rate(lerp(0.9, 1.1, time.elapsed_seconds().fract()) as f64)
            .handle()
        ]})
        .with_children(|parent|{
            parent.spawn(ParticleEffectBundle {
                effect: ParticleEffect::new(effect_bundle.particle_hit.clone()),
                ..Default::default()
            });

            parent.spawn((
                MaterialMeshBundle {
                    mesh: effect_bundle.mesh_quad.clone(),
                    material: effect_bundle.material_ring.clone(),
                    transform: Transform::from_xyz(0.0, 0.2, 0.0)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ..Default::default()
                },
                bevy::pbr::NotShadowCaster,
                bevy::pbr::NotShadowReceiver,
                ColorUniform::from(Color::WHITE),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::rgba(4.0,6.0,2.0,1.0).into(), 0.0, Ease::Linear),
                    (Color::NONE.into(), 0.4, Ease::Out(SimpleCurve::Offset(0.8))),
                ], 0)),
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::ZERO.into(), 0.0, Ease::Linear),
                    (Vec3::splat(2.0).into(), 0.4, Ease::Out(SimpleCurve::Power(2))),
                ], 0)),
            ));

            parent.spawn((
                SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.5, 0.0).with_scale(Vec3::ZERO)),
                effect_bundle.mesh_sphere.clone(),
                effect_bundle.material_warp.clone(),
                bevy::pbr::NotShadowCaster,
                bevy::pbr::NotShadowReceiver,
                ColorUniform::from(Color::WHITE),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::WHITE.into(), 0.0, Ease::Linear),
                    (Color::NONE.into(), 0.5, Ease::Out(SimpleCurve::Sine)),
                ], 0)),
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::ZERO.into(), 0.0, Ease::Linear),
                    (Vec3::splat(3.0).into(), 0.5, Ease::Out(SimpleCurve::Power(2))),
                ], 0)),
            ));

            parent.spawn((
                MaterialMeshBundle {
                    mesh: effect_bundle.mesh_fuzz.clone(),
                    material: effect_bundle.material_vines.clone(),
                    transform: Transform::default().with_scale(Vec3::ZERO),
                    ..Default::default()
                },
                bevy::pbr::NotShadowCaster,
                bevy::pbr::NotShadowReceiver,
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::ZERO.into(), 0.0, Ease::Linear),
                    (Vec3::splat(2.0).into(), 0.2, Ease::Out(SimpleCurve::Power(2))),
                    (Vec3::ZERO.into(), 0.6, Ease::In(SimpleCurve::Power(1))),
                ], 0)),
            ));
        });
    }
}