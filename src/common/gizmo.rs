use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    utils::hashbrown::HashMap,
};

//https://github.com/lassade/bevy_gizmos/tree/for-bevy-master-branch/src

#[derive(Default, Resource, Debug)]
pub struct GizmoBuffer {
    commands: Vec<Gizmo>,
    entities: Vec<Entity>,
    materials: HashMap<u32, Handle<StandardMaterial>>,
    mesh_sphere: Handle<Mesh>,
    mesh_cube: Handle<Mesh>,
}

impl GizmoBuffer {
    pub fn draw(&mut self, gizmo: Gizmo){
        self.commands.push(gizmo);
    }
}

#[derive(Clone, Default, Debug)]
pub struct LineStrip(Vec<Vec3>);
impl From<Vec<Vec3>> for LineStrip { fn from(list: Vec<Vec3>) -> Self{ Self(list) } }
impl From<LineStrip> for Mesh {
    fn from(strip: LineStrip) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        let vertices: Vec<[f32; 3]> = strip.0.iter().map(|point| [point.x,point.y,point.z]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,vec![[0.0, 1.0, 0.0]; strip.0.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; strip.0.len()]);
        let indices: Vec<u16> = (0..strip.0.len() as u16).collect();
        mesh.set_indices(Some(Indices::U16(indices)));
        mesh
    }
}

#[derive(Component, Clone, Debug)]
pub enum Gizmo {
    Sphere {
        position: Vec3,
        radius: f32,
        color: Color
    },
    Lines {
        line: LineStrip,
        color: Color
    },
}

fn setup_gizmo_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut gizmos: ResMut<GizmoBuffer>
){
    gizmos.mesh_sphere = meshes.add(Mesh::from(shape::Icosphere::default()));
    gizmos.mesh_cube = meshes.add(Mesh::from(shape::Cube::default()));
}

fn cleanup_gizmo_system(
    mut commands: Commands,
    mut gizmos: ResMut<GizmoBuffer>,
) {
    while let Some(entity) = gizmos.entities.pop() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_gizmo_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut gizmos: ResMut<GizmoBuffer>
) {
    while let Some(command) = gizmos.commands.pop() {
        let (mesh, color, transform) = match command {
            Gizmo::Sphere { position, radius, color } => {
                (gizmos.mesh_sphere.clone(), color, Transform::from_scale(Vec3::splat(radius)).with_translation(position))
            },
            Gizmo::Lines { line, color } => {
                let mesh = Mesh::from(line);
                let handle = meshes.add(mesh);
                (handle, color, Transform::IDENTITY)
            }
        };


        let material = if let Some(material_handle) = gizmos.materials.get(&color.as_linear_rgba_u32()) {
            material_handle.clone()
        } else {
            let material_handle = materials.add(bevy::pbr::StandardMaterial {
                base_color: color, unlit: true, ..Default::default()
            });
            gizmos.materials.insert(color.as_linear_rgba_u32(), material_handle.clone());
            material_handle
        };

        let entity = commands.spawn((
            PbrBundle { transform, mesh, material, ..Default::default() },
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver
        ));
        gizmos.entities.push(entity.id());
    }
}

pub struct GizmosPlugin; impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        static DEBUG: &str = "debug";
        app.add_stage_after(CoreStage::Update, DEBUG, SystemStage::parallel());

        app.init_resource::<GizmoBuffer>();
        app.add_startup_system(setup_gizmo_system);
        app.add_system_to_stage(CoreStage::PreUpdate, cleanup_gizmo_system);
        app.add_system_to_stage(DEBUG, update_gizmo_system);
    }
}