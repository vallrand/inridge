use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::common::animation::ease::{lerp, Ease, SimpleCurve};
use crate::common::animation::{AnimationTimeline, Animator, Track, TransformScale, TransformTranslation};
use crate::common::loader::AssetBundle;
use crate::logic::CombatEvent;
use crate::extensions::EntityLookupTable;
use crate::logic::{MilitaryBinding,TrajectoryEffect};
use crate::materials::{ProjectileTrailMaterial, StripeMesh, ColorUniform};
use crate::scene::{EffectAssetBundle, AudioAssetBundle};

pub fn animate_projectile_effect(
    mut commands: Commands,
    mut events: EventReader<CombatEvent>,
    query_unit: Query<(Entity, &EntityLookupTable, &MilitaryBinding, &GlobalTransform), Changed<MilitaryBinding>>,
    query_transform: Query<(&Transform, &GlobalTransform)>,

    audio: Res<Audio>,
    audio_bundle: Res<AssetBundle<AudioAssetBundle>>,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    mut materials: ResMut<Assets<ProjectileTrailMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,

    query_effect: Query<(Entity, Ref<TrajectoryEffect>, &Handle<ProjectileTrailMaterial>)>,
){
    for (_entity, effect, material) in query_effect.iter() {
        let material = materials.get_mut(material).unwrap();
        material.uv_transform.y = lerp(1.0, -material.uv_transform.w + 1.0, effect.intro.percent());
        material.uv_transform.y = lerp(material.uv_transform.y, -material.uv_transform.w, effect.outro.percent());
    }

    for event in events.iter() {
        let &CombatEvent::ProjectileLaunch(effect_entity, origin_entity, target_entity) = event else { continue };
        let Ok((
            entity, lookup, military, global_transform
        )) = query_unit.get(origin_entity) else { continue };

        let Some(origin_transform) = (match military {
            MilitaryBinding::Trajectory { key, axis, .. } =>
            lookup.get(key).and_then(|&entity|query_transform.get_component::<GlobalTransform>(entity).ok())
            .map(|transform|{
                let mut transform = transform.compute_matrix();
                if let Some(matrix) = axis {
                    transform = transform * Mat4::from_mat3(matrix.inverse())
                }
                transform
            }),
            _ => query_transform.get_component::<GlobalTransform>(entity).ok().map(|transform|transform.compute_matrix()),
        }) else { continue };
        let Ok(target_transform) = query_transform.get_component::<GlobalTransform>(target_entity) else { continue };

        let trail_width = 0.4;
        let distance = target_transform.translation().distance(origin_transform.transform_point3(Vec3::ZERO));

        commands.entity(effect_entity).insert(MaterialMeshBundle{
            material: materials.add(ProjectileTrailMaterial{
                color: Color::rgb(1.0,0.0,0.0), head_color: Color::rgba(0.6,0.7,0.2,0.5),
                billboard: true, blend_mode: AlphaMode::Blend,
                uv_transform: Vec4::new(0.0, 1.0, 1.0, distance),
                vertical_fade: Vec2::new(0.5 / distance, 1.0), iterations: 4, time_scale: 1.0,
                ..Default::default()
            }),
            mesh: meshes.add(StripeMesh::from_arc(&GlobalTransform::from(origin_transform), target_transform)
            .with_quality(16).with_stroke(trail_width)
            .into()),
            ..Default::default()
        }).insert((
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));

        commands.spawn(SpatialBundle::from_transform(
            Transform::from_matrix(global_transform.compute_matrix().inverse() * origin_transform)
        )).insert(AnimationTimeline::default().with_cleanup(0.8))
        .insert(AudioEmitter{instances:vec![
            audio.play(audio_bundle.launch_projectile.clone()).handle()
        ]})
        .with_children(|parent|{
            parent.spawn(MaterialMeshBundle {
                mesh: effect_bundle.mesh_tube.clone(),
                material: effect_bundle.material_reload.clone(),
                transform: Transform::default()
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::ZERO),
                ..Default::default()
            }).insert((
                bevy::pbr::NotShadowCaster,
                bevy::pbr::NotShadowReceiver,
            )).insert((
                ColorUniform::from(Color::NONE),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::NONE.into(), 0.0, Ease::Linear),
                    (Color::WHITE.into(), 0.3, Ease::Out(SimpleCurve::Power(2))),
                    (Color::NONE.into(), 0.6, Ease::In(SimpleCurve::Power(1))),
                ], 0)),
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::new(0.0, 2.0, 0.0).into(), 0.0, Ease::Linear),
                    (Vec3::new(0.4, 0.8, 0.4).into(), 0.4, Ease::Out(SimpleCurve::Power(3))),
                ], 0)),
                Animator::<TransformTranslation>::new().add(Track::from_frames(vec![
                    (Vec3::new(1.0, 0.0, 0.0).into(), 0.0, Ease::Linear),
                    (Vec3::ZERO.into(), 0.6, Ease::Out(SimpleCurve::Sine)),
                ], 0)),
            ));

            parent.spawn((
                SpatialBundle::from_transform(Transform::default()
                    .with_translation(Vec3::new(0.6, 0.0, 0.0))
                    .with_scale(Vec3::ZERO)
                ),
                effect_bundle.mesh_quad.clone(),
                effect_bundle.material_flash.clone(),
                bevy::pbr::NotShadowCaster,
                bevy::pbr::NotShadowReceiver,
                ColorUniform::from(Color::NONE),
                Animator::<TransformScale>::new().add(Track::from_frames(vec![
                    (Vec3::ZERO.into(), 0.0, Ease::Linear),
                    (Vec3::splat(2.0).into(), 0.3, Ease::Out(SimpleCurve::Power(3))),
                ], 0)),
                Animator::<ColorUniform>::new().add(Track::from_frames(vec![
                    (Color::rgba(12.0,16.0,2.0,1.0).into(), 0.0, Ease::Linear),
                    (Color::NONE.into(), 0.3, Ease::In(SimpleCurve::Sine)),
                ], 0)),
            ));
        }).set_parent(entity);
    }
}

pub fn reorient_targeting_systems(
    query_unit: Query<(Entity, &EntityLookupTable, &MilitaryBinding, &GlobalTransform), Changed<MilitaryBinding>>,
    mut query_transform: Query<(&Parent, &mut Transform, &GlobalTransform)>,
){
    for (_entity, lookup, military, transform) in query_unit.iter() {
        let MilitaryBinding::Trajectory { key, orientation, axis, .. } = military else { continue };

        let Some(entity) = lookup.get(key) else { continue };
        let Ok(parent_transform) = query_transform.get_component::<Parent>(*entity)
        .and_then(|parent|query_transform.get_component::<GlobalTransform>(parent.get())) else { continue };
        
        let parent_rotation = Quat::from_mat4(&(transform.compute_matrix().inverse() * parent_transform.compute_matrix())).inverse();

        let Ok(mut local_transform) = query_transform.get_component_mut::<Transform>(*entity) else { continue };

        let local_rotation = axis.as_ref().map_or(Quat::IDENTITY, |mat| Quat::from_mat3(mat));

        local_transform.rotation = parent_rotation.mul_quat(orientation.clone()).mul_quat(local_rotation);
    }
}