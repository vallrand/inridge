use std::f32::consts::FRAC_PI_2;
use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::common::animation::ease::{Ease, SimpleCurve};
use crate::logic::{GroupLink, MilitaryBinding, MilitarySupply};
use crate::materials::{HairBall, ColorUniform};
use crate::scene::EffectAssetBundle;

#[derive(Component, Clone)]
pub struct DomeEffect {
    pub entity: Entity,
    pub barrier: Entity,
    pub lightning: Entity,
    pub intensity: f32,

    pub intro_duration: f32,
    pub outro_duration: f32,
    pub ease: Ease,
}

pub fn animate_dome_effect(
    time: Res<Time>,
    mut commands: Commands,
    mut query_unit: ParamSet<(
        Query<(Entity, Option<&mut DomeEffect>, &MilitaryBinding, &MilitarySupply, &GlobalTransform), With<GroupLink>>,
        Query<(Entity, &mut DomeEffect), Or<(Without<MilitarySupply>, Without<GroupLink>)>>,
    )>,
    mut query_uniform: Query<&mut ColorUniform>,
    mut query_transform: Query<&mut Transform>,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    mut meshes: ResMut<Assets<Mesh>>,
){
    for (entity, mut effect) in query_unit.p1().iter_mut() {
        effect.intensity = (effect.intensity - time.delta_seconds() / effect.outro_duration).max(0.0);

        if let Ok(mut transform) = query_transform.get_mut(effect.barrier) {
            transform.scale = Vec3::ZERO.lerp(Vec3::ONE, effect.ease.calculate(effect.intensity));
        }
        if let Ok(mut uniform) = query_uniform.get_mut(effect.lightning) {
            uniform.color.set_a(effect.intensity);
        }

        if effect.intensity <= 0.0 {
            commands.entity(effect.entity).despawn_recursive();
            commands.entity(entity).remove::<DomeEffect>();
        }
    }
    for (entity, effect, military, supply, transform) in query_unit.p0().iter_mut() {
        let MilitaryBinding::Area { .. } = military else { continue };
        let radius = military.radius() * supply.range_multipler();

        if let Some(mut effect) = effect {
            effect.intensity = (effect.intensity + time.delta_seconds() / effect.intro_duration).min(1.0);

            if let Ok(mut transform) = query_transform.get_mut(effect.barrier) {
                transform.scale = Vec3::ZERO.lerp(Vec3::ONE, effect.ease.calculate(effect.intensity));
            }
            if let Ok(mut uniform) = query_uniform.get_mut(effect.lightning) {
                uniform.color.set_a(effect.intensity);
            }

            let Ok(mut transform) = query_transform.get_mut(effect.entity) else { continue };
            let target_scale = Vec3::splat(2.0 * radius);
            if transform.scale.abs_diff_eq(target_scale, f32::EPSILON) { continue; }
            transform.scale = transform.scale.lerp(target_scale, 0.1);
            continue;
        }

        let effect = commands.spawn(
            SpatialBundle::from_transform(Transform::from_matrix(transform.compute_matrix())
            .with_translation(transform.transform_point(Vec3::Y))
            .with_scale(Vec3::splat(2.0 * radius))
        )).id();
        let lightning = commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(HairBall {
                seed: time.elapsed().subsec_nanos(), radius: 0.5, width: 0.1, quantity: 16, hemisphere: false
            }.into()),
            material: effect_bundle.material_lightning.clone(),
            ..Default::default()
        }).insert((
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ))
        .insert(ColorUniform::from(Color::rgba(1.0,1.0,1.0,0.0)))
        .set_parent(effect).id();
        let barrier = commands.spawn(MaterialMeshBundle {
            mesh: effect_bundle.mesh_sphere.clone(),
            material: effect_bundle.material_barrier.clone(),
            transform: Transform::default()
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2))
                .with_scale(Vec3::ZERO),
            ..Default::default()
        }).insert((
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        )).set_parent(effect).id();

        commands.entity(entity).insert(DomeEffect {
            entity: effect,
            lightning, barrier,
            intensity: 0.0,
            ease: Ease::Out(SimpleCurve::Circular),
            intro_duration: 1.0, outro_duration: 0.2,
        });
    }
}