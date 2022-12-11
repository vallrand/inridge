use bevy::prelude::*;
use crate::common::*;

pub fn setup_level_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    commands.spawn((
        PbrBundle{
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.5, rings: 4*64, depth: 6.0, ..Default::default()
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        spline::MeshDeformation::default(),
        spline::Spline::<spline::ControlPoint>::new(vec![
            spline::ControlPoint{
                position: Vec3::new(0.0, 0.0, 0.0), ..Default::default()
            },
            spline::ControlPoint{
                position: Vec3::new(4.0, -1.0, 0.0), ..Default::default()
            },
            spline::ControlPoint{
                position: Vec3::new(5.0, 4.0, 1.0), ..Default::default()
            },
            spline::ControlPoint{
                position: Vec3::new(6.0, 2.0, -1.0), ..Default::default()
            },
        ], 4),
    ));
}

pub fn update_level_system(
    mut gizmos: ResMut<gizmo::GizmoBuffer>,
    mut query: Query<(&mut spline::Spline::<spline::ControlPoint>)>
){
    //gizmos.draw(gizmo::Gizmo::Sphere { radius: (5.0), color: Color::rgb(0.4, 0.4, 1.0) });

    gizmos.draw(gizmo::Gizmo::Lines { line: gizmo::LineStrip::from(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(-4.0,0.0,0.0)
        ]
    ), color: Color::rgb(1.0,0.0,0.0) });

    for (mut spline) in query.iter_mut() {
        spline.recalculate_length();
        static segments: usize = 20;
        let mut points: Vec<Vec3> = Vec::with_capacity(segments);
        for i in 0..segments {
            let t = i as f32 / segments as f32 * spline.nodes.len() as f32;
            //spline.normalize(i as f32 / 100.0);
            let transform = spline.sample(t);
            points.push(transform.translation);
        }

        gizmos.draw(gizmo::Gizmo::Lines {
            line: gizmo::LineStrip::from(points),
            color: Color::rgb(1.0, 0.0, 0.0),
        })
    }
}