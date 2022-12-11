use bevy::prelude::*;

#[derive(Clone, Default, Component)]
pub struct Spline<T> {
    pub nodes: Vec<T>,
    pub resolution: usize,
    dirty: bool,
    arclengths: Vec<f32>
}

pub enum ControlVariant {
    Tension(f32),
    Direction(Vec3)
}
pub struct ControlPoint {
    pub position: Vec3,
    pub scale: Vec2,
    pub roll: f32,
    pub variant: ControlVariant
}
impl Default for ControlPoint {
    fn default() -> Self { Self { position: Vec3::ZERO, scale: Vec2::ONE, roll: 0.0, variant: ControlVariant::Tension(0.5) } }
}

impl<T> std::ops::Deref for Spline<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target { &self.nodes }
}
impl<T> std::ops::DerefMut for Spline<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.nodes
    }
}

impl Spline<ControlPoint> {
    pub fn length(&self) -> f32 { *self.arclengths.last().unwrap_or(&0.0) }
    pub fn new(nodes: Vec<ControlPoint>, resolution: usize) -> Self {
        Self { nodes, resolution, dirty: true, arclengths: Vec::new() }
    }
    pub fn sample(&self, time: f32) -> Transform {
        let index: usize = (time.trunc() as usize).clamp(0, self.nodes.len() - 1);
        let fraction = time.fract();

        let prev = &self.nodes[index];
        let next = &self.nodes[(index+1).min(self.nodes.len() - 1)];

        let (c0, c1) = match prev.variant {
            ControlVariant::Direction(direction) => (prev.position, prev.position + direction),
            ControlVariant::Tension(tension) => {
                let prev_prev = &self.nodes[index.max(1)-1];
                let tangent = (next.position - prev_prev.position) * tension;
                (prev.position, prev.position + tangent / 3.0)
            }
        };
        let (c2, c3) = match next.variant {
            ControlVariant::Direction(direction) => (next.position - direction, next.position),
            ControlVariant::Tension(tension) => {
                let next_next = &self.nodes[(index+2).min(self.nodes.len() - 1)];
                let tangent = (next_next.position - prev.position) * tension;
                (next.position - tangent / 3.0, next.position)
            }
        };

        let translation = crate::common::ease::cubic_bezier(c0, c1, c2, c3, fraction);
        let tangent = crate::common::ease::cubic_bezier_derivative(c0, c1, c2, c3, fraction).normalize();
        
        let roll = prev.roll + (next.roll - prev.roll) * fraction;
        let binormal = Vec3::cross(tangent, Quat::from_axis_angle(Vec3::Z, roll) * Vec3::Y).normalize();
        let normal = Vec3::cross(tangent, binormal);
        
        let rotation = Quat::from_mat3(&Mat3::from_cols(binormal, normal, tangent)).normalize();
        let scale = prev.scale.lerp(next.scale, fraction);
        
        Transform{ translation, rotation, scale: Vec3::new(scale.x, scale.y, 1.0) }
    }
    pub fn recalculate_length(&mut self){
        let samples = self.resolution * (self.nodes.len() - 1) + 1;
        self.arclengths.resize(samples, default());

        let mut prev = self.sample(0.0).translation;
        let mut length: f32 = 0.0;
        let step: f32 = (self.resolution as f32).recip();
        for i in 0..samples {
            let next = self.sample(i as f32 * step).translation;
            length += (prev - next).length();
            self.arclengths[i] = length;
            prev = next;
        }
        self.dirty = false;
    }
    pub fn normalize(&self, u: f32) -> f32 {
        let length: f32 = u * self.length();
        let mut low: usize = 0;
        let mut high: usize = self.arclengths.len();
        let mut index: usize = 0;
        while low < high {
            index = (low + high) / 2;
            if self.arclengths[index] < length {
                low = index + 1
            } else {
                high = index
            }
        }
        if index == self.arclengths.len() - 1 {
            index -= 1;
        }
        let l0 = self.arclengths[index];
        self.nodes.len() as f32 *
        (index as f32 + (length - l0) / (self.arclengths[index + 1] - l0)) / self.arclengths.len() as f32
    }
}

#[derive(Clone, Default, Component)]
pub struct MeshDeformation {
    pub limit: u16,
    pub offset: f32,
    vertices: Vec<(f32,Vec3,Vec3)>
}
impl From<&Mesh> for MeshDeformation {
    fn from(mesh: &Mesh) -> Self {
        let aabb = mesh.compute_aabb().unwrap();

        let attribute_position = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let attribute_normal = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap();

        let normals = attribute_normal.as_float3().unwrap();
        let positions = attribute_position.as_float3().unwrap();

        let mut vertices = Vec::with_capacity(positions.len());
        for (vertex_position, vertex_normal) in positions.iter().zip(normals.iter()) {
            let mut position = Vec3::from(*vertex_position);
            let mut normal = Vec3::from(*vertex_normal);

            position.y -= aabb.min().y;
            position = Quat::from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2) * position;
            normal = Quat::from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2) * normal;

            let distance = position.z;
            position.z = 0.0;

            vertices.push((distance, position, normal));
        }

        Self { limit: 1, offset: 0.0, vertices }
    }
}

impl MeshDeformation {
    pub fn apply(&self, spline: &Spline<ControlPoint>, mesh: &mut Mesh){
        let mut positions: Vec<Vec3> = Vec::with_capacity(self.vertices.len());
        let mut normals: Vec<Vec3> = Vec::with_capacity(self.vertices.len());
        for (distance, position, normal) in self.vertices.iter() {
            let transform = spline.sample(spline.normalize(*distance));

            positions.push(transform.transform_point(position.clone()));
            normals.push((transform.rotation * normal.clone()).normalize());
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }
}

fn update_spline_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&mut Handle<Mesh>, &mut MeshDeformation, &mut Spline<ControlPoint>), Changed<Spline<ControlPoint>>>
){
    for (mut handle, mut deform, mut spline) in query.iter_mut() {
        if deform.vertices.is_empty() {
            deform.clone_from(&MeshDeformation::from(meshes.get(&handle).unwrap()));
        }
        
        spline.recalculate_length();

        let mesh = meshes.get_mut(&handle).unwrap();
        deform.apply(&spline, mesh);
    }
}

pub struct SplinePlugin;
impl Plugin for SplinePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_spline_system);
    }
}