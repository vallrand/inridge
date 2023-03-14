use bevy::prelude::*;
use std::f32::consts::TAU;
use super::StageBlueprint;

#[derive(Component, Clone, Default)]
pub struct OrbitingTransform {
    pub center: Vec3,
    pub normal: Vec3,
    pub forward: Vec3,
    pub distance: f32,
    pub timer: Timer,
}

pub fn setup_lighting(commands: &mut Commands, stage: &StageBlueprint){
    for area in stage.areas.iter() {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 6.0 * 1500.0,
                range: area.radius() * 8.0,
                shadows_enabled: true,
                color: Color::rgb(0.95, 0.85, 1.0),
                ..Default::default()
            },
            ..Default::default()
        }).insert(OrbitingTransform {
            center: area.center,
            normal: Vec3::Y,
            forward: Vec3::any_orthonormal_vector(&Vec3::Y),
            distance: area.radius() * 4.0,
            timer: Timer::from_seconds(60.0, TimerMode::Repeating),
        });
        let mut timer = Timer::from_seconds(60.0, TimerMode::Repeating);
        timer.tick(std::time::Duration::from_secs_f32(30.0));
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 800.0,
                range: 20.0,
                shadows_enabled: false,
                color: Color::rgb(0.8, 1.0, 0.9),
                ..Default::default()
            },
            ..Default::default()
        }).insert(OrbitingTransform {
            center: area.center,
            normal: Vec3::Y,
            forward: Vec3::any_orthonormal_vector(&Vec3::Y),
            distance: area.radius() * 2.0,
            timer,
        });
    }
}

pub fn update_orbiting_transforms(
    time: Res<Time>,
    mut query: Query<(&mut OrbitingTransform, &mut Transform)>
){
    for (mut orbit, mut transform) in query.iter_mut() {
        orbit.timer.tick(time.delta());
        transform.translation = orbit.center;
        transform.translation += 
        Quat::from_axis_angle(orbit.normal, orbit.timer.percent() * TAU)
        .mul_vec3(orbit.distance * orbit.forward);
    }
}