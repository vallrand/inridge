use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use std::f32::consts::FRAC_PI_2;
use crate::common::loader::AssetBundle;
use crate::common::animation::{Animator, Track, TransformScale, AnimationTimeline};
use crate::common::animation::ease::{Ease, SimpleCurve, lerp};
use crate::materials::{ColorUniform, ModelEffectLayeredMaterial};
use crate::scene::{EffectAssetBundle, UnitBlueprint};
use crate::logic::{Integrity, CombatEvent, SourceLink};

pub fn animate_destruction(
    mut events: EventReader<CombatEvent>,
    mut commands: Commands,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    query_unit: Query<(&mut Transform, &GlobalTransform, Option<&SourceLink>)>,
    children: Query<&Children>,
    query: Query<(Entity, &AnimationTimeline), (Without<Integrity>, With<Handle<UnitBlueprint>>)>,
    mut query_uniform: Query<&mut ColorUniform, With<Handle<ModelEffectLayeredMaterial>>>,
){
    for event in events.iter() {
        let &CombatEvent::Destruct(entity) = event else { continue };
        let Ok((local_transform, transform, source)) = query_unit.get(entity) else { continue };

        commands.entity(entity)
        .insert(AnimationTimeline::default().with_cleanup(2.4))
        .insert(Animator::<TransformScale>::new()
            .add(Track::from_frames(vec![
                (local_transform.scale.into(), 0.0, Ease::Linear),
                (Vec3::ZERO.into(), 0.8, Ease::In(SimpleCurve::Power(1))),
                (Vec3::ZERO.into(), 2.4, Ease::Linear),
            ], 0)));
        
        if source.is_some() { continue; }
        commands.spawn(SpatialBundle::from_transform(Transform::from_matrix(transform.compute_matrix())))
        .insert(AnimationTimeline::default().with_cleanup(2.4))
        .with_children(|parent|{
            parent.spawn((
                SpatialBundle::from_transform(Transform::default()
                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
                    .with_translation(Vec3::new(0.0, 0.25, 0.0))),
                effect_bundle.mesh_quad.clone(),
                effect_bundle.material_wave.clone(),
                ColorUniform::from(Color::WHITE),
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::ZERO.into(), 0.0, Ease::Linear),
                    (Vec3::ZERO.into(), 0.1, Ease::Linear),
                    (Vec3::splat(8.0).into(), 0.9, Ease::Out(SimpleCurve::Power(2))),
                ], 0)),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::WHITE.into(), 0.0, Ease::Linear),
                    (Color::WHITE.into(), 0.1, Ease::Linear),
                    (Color::NONE.into(), 0.9, Ease::In(SimpleCurve::Power(1))),
                ], 0)),
            ));
    
            parent.spawn((
                SpatialBundle::from_transform(Transform::default()
                    .with_translation(Vec3::new(0.0, 0.5, 0.0))
                    .with_scale(Vec3::splat(4.0))
                ),
                effect_bundle.mesh_quad.clone(),
                effect_bundle.material_explosion_fire.clone(),
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
                ColorUniform::from(Color::WHITE),
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::ZERO.into(), 0.0, Ease::Linear),
                    (Vec3::splat(3.0).into(), 0.5, Ease::Out(SimpleCurve::Power(3))),
                ], 0)),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::rgba(16.0,8.0,4.0,1.0).into(), 0.0, Ease::Linear),
                    (Color::NONE.into(), 0.5, Ease::Out(SimpleCurve::Sine)),
                ], 0)),
            ));
    
            parent.spawn(ParticleEffectBundle {
                effect: ParticleEffect::new(effect_bundle.particle_explosion.clone()),
                transform: Transform::from_xyz(0.0, 0.2, 0.0),
                ..Default::default()
            });
        });
    }
    for (entity, animation) in query.iter() {
        let percent = lerp(1.0, 16.0, (animation.elapsed.as_secs_f32() / 0.96).min(1.0));
        for entity in children.iter_descendants(entity) {
            let Ok(mut uniform) = query_uniform.get_mut(entity) else { continue };
            uniform.color.set_g(percent);
        }
    }
}