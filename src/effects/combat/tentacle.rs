use bevy::prelude::*;
use crate::common::loader::AssetBundle;
use crate::common::spline;
use crate::scene::EffectAssetBundle;
use crate::logic::{MilitaryBinding,TargetLock,SourceLink,TrajectoryEffect};
use crate::materials::ColorUniform;

fn recalculate_tentacle_path(origin_transform: &Mat4, target_transform: &Mat4) -> Vec<spline::ControlPoint> {
    let origin_position = origin_transform.transform_point3(Vec3::Y * 1.0);
    let target_position = target_transform.transform_point3(Vec3::Y * 0.5);
    let distance = target_position.distance(origin_position);

    let origin_normal = 0.5 * distance * origin_transform.transform_vector3(Vec3::Y);
    let origin_control_point = spline::ControlPoint {
        position: origin_position, scale: Vec2::ONE,
        variant: spline::ControlVariant::Direction(origin_normal),
        ..Default::default()
    };

    let target_normal = -0.25 * distance * target_transform.transform_vector3(Vec3::Y);
    let target_control_point = spline::ControlPoint {
        position: target_position, scale: Vec2::splat(0.1),
        variant: spline::ControlVariant::Direction(target_normal),
        ..Default::default()
    };
    
    vec![origin_control_point, target_control_point]
}

pub fn animate_tentacle_effect(
    mut commands: Commands,
    effect_bundle: Res<AssetBundle<EffectAssetBundle>>,
    mut query_effect: ParamSet<(
        Query<(Entity, &SourceLink, &TargetLock), (With<TrajectoryEffect>, Without<spline::Spline::<spline::ControlPoint>>)>,
        Query<(&SourceLink, &TargetLock, &TrajectoryEffect, &mut spline::Spline::<spline::ControlPoint>, &mut spline::MeshDeformation, &mut ColorUniform)>
    )>,
    query_unit: Query<&MilitaryBinding>,
    transforms: Query<Ref<GlobalTransform>>,
    mut meshes: ResMut<Assets<Mesh>>,
){
    for (
        source, target, trajectory, mut spline, mut deformation, mut uniform
    ) in query_effect.p1().iter_mut() {
        let percent = (trajectory.intro.percent() - trajectory.outro.percent()).clamp(0.0, 1.0);
        deformation.offset = percent - 1.0;
        uniform.color.set_r(percent);

        let Ok(origin_transform) = transforms.get(**source) else { continue };
        let Ok(target_transform) = transforms.get(**target) else { continue };
        if origin_transform.is_changed() || target_transform.is_changed() {
            let target_transform = origin_transform.compute_matrix().inverse() * target_transform.compute_matrix();
            let control_points = recalculate_tentacle_path(&Mat4::IDENTITY, &target_transform);
            spline.nodes = control_points;
            spline.dirty = true;
        }
    }
    for (entity, source, target) in query_effect.p0().iter_mut() {
        let Ok(origin_transform) = transforms.get(**source) else { continue };
        let Ok(target_transform) = transforms.get(**target) else { continue };
        let Ok(MilitaryBinding::Connection { .. }) = query_unit.get(**source) else { continue };

        let transform = origin_transform.compute_matrix();
        let target_transform = transform.inverse() * target_transform.compute_matrix();

        let control_points = recalculate_tentacle_path(&Mat4::IDENTITY, &target_transform);

        commands.entity(entity).insert((
            MaterialMeshBundle{
                mesh: meshes.add(shape::Cylinder{ radius: 0.04, height: 1.0, resolution: 8, segments: 32 }.into()),
                material: effect_bundle.material_tentacle.clone(),
                transform: Transform::from_matrix(transform),
                ..Default::default()
            },
            ColorUniform::from(Color::NONE),
            spline::MeshDeformation::default().with_range(true, 0.0..1.0).with_offset(-1.0),
            spline::Spline::<spline::ControlPoint>::new(control_points, 16),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }
}