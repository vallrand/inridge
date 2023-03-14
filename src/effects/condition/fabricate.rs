use bevy::prelude::*;
use crate::effects::animation::MovementFormation;
use crate::common::animation::ease::{lerp, Ease, SimpleCurve};
use crate::materials::{ColorUniform, ModelEffectLayeredMaterial};
use crate::logic::{Integrity, GroupLink};

pub struct FabricationAnimation {
    ease: Ease,
    entity: Entity,
    target_scale: Vec3,
    timer: Timer,
}

pub fn animate_unit_condition_fabricated(
    time: Res<Time>,
    mut query_unit: Query<(Entity, &mut Transform), (With<MovementFormation>, Added<Integrity>, Without<GroupLink>)>,
    children: Query<&Children>,
    mut query_uniform: Query<&mut ColorUniform, With<Handle<ModelEffectLayeredMaterial>>>,
    mut transitions: Local<Vec<FabricationAnimation>>
){
    for (entity, transform) in query_unit.iter() {
        transitions.push(FabricationAnimation {
            entity, ease: Ease::In(SimpleCurve::Power(1)), target_scale: transform.scale,
            timer: Timer::from_seconds(4.0, TimerMode::Once)
        });
    }
    transitions.retain_mut(|animation|{
        if let Ok(mut transform) = query_unit.get_component_mut::<Transform>(animation.entity) {
            let percent = (2.0 * animation.timer.percent()).clamp(0.0, 1.0);
            transform.scale = Vec3::lerp(Vec3::ZERO, animation.target_scale, percent);
        }
        
        let percent = animation.ease.calculate(animation.timer.percent());
        for entity in children.iter_descendants(animation.entity) {
            let Ok(mut uniform) = query_uniform.get_mut(entity) else { continue };
            uniform.color.set_a(lerp(4.0, 1.0, percent));
        }
        
        let finished = animation.timer.finished();
        animation.timer.tick(time.delta());
        !finished
    });
}