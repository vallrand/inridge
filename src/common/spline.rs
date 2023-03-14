use std::ops::Range;
use bevy::prelude::*;

#[derive(Clone, Default, Component)]
pub struct Spline<T> {
    pub nodes: Vec<T>,
    pub resolution: usize,
    pub dirty: bool,
    arclengths: Vec<f32>
}
impl<T> From<Vec<T>> for Spline<T> {
    fn from(nodes: Vec<T>) -> Self { Self { resolution: 1, dirty: true, arclengths: Vec::with_capacity(nodes.len()), nodes } }
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
    pub fn with_resolution(mut self, resolution: usize) -> Self { self.resolution = resolution; self }
    pub fn from_positions(positions: &[Vec3], resolution: usize) -> Self { Self {
        nodes: positions.iter().map(|&position|
            ControlPoint{ position, ..Default::default() }
        ).collect(),
        resolution, dirty: true, arclengths: Vec::new()
    } }
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

        let translation = crate::common::animation::ease::cubic_bezier(c0, c1, c2, c3, fraction);
        let tangent = crate::common::animation::ease::cubic_bezier_derivative(c0, c1, c2, c3, fraction).normalize();
        
        let roll = prev.roll + (next.roll - prev.roll) * fraction;
        let binormal = Vec3::cross(tangent, Quat::from_axis_angle(Vec3::Z, roll) * Vec3::Y).normalize();
        let normal = Vec3::cross(tangent, binormal);
        
        let rotation = Quat::from_mat3(&Mat3::from_cols(binormal, normal, tangent)).normalize();
        let scale = prev.scale.lerp(next.scale, fraction);
        
        Transform{ translation, rotation, scale: Vec3::new(scale.x, scale.y, 1.0) }
    }
    pub fn recalculate_length(&mut self){
        if !self.dirty { return; }
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
        (index as f32 + (length - l0) / (self.arclengths[index + 1] - l0)) *
        (self.nodes.len() - 1) as f32 / (self.arclengths.len() - 1) as f32
    }
}

#[derive(Clone, Default, Component)]
pub struct MeshDeformation {
    pub range: Option<Range<f32>>,
    pub offset: f32,
    pub stretch: bool,
    pub handle: Option<Handle<Mesh>>,
    vertices: Vec<(f32,Vec3,Vec3)>,
}

impl MeshDeformation {
    pub fn with_range(mut self, stretch: bool, range: Range<f32>) -> Self { self.stretch = stretch; self.range = Some(range); self }
    pub fn with_offset(mut self, offset: f32) -> Self { self.offset = offset; self }
    pub fn cache_mesh(&mut self, mesh: &Mesh){
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

        self.vertices = vertices;
    }
    pub fn apply(&self, spline: &Spline<ControlPoint>, mesh: &mut Mesh){
        let mut positions: Vec<[f32;3]> = Vec::with_capacity(self.vertices.len());
        let mut normals: Vec<[f32;3]> = Vec::with_capacity(self.vertices.len());
        let segments = spline.nodes.len() as f32 - 1.0;
        for &(distance, position, normal) in self.vertices.iter() {
            let mut height: f32 = self.offset * segments + if self.stretch { spline.normalize(distance) }else{ distance };
            if let Some(range) = self.range.as_ref() { height = height.clamp(range.start * segments, range.end * segments); }
            let transform = spline.sample(height);

            positions.push(transform.transform_point(position).to_array());
            normals.push((transform.rotation * normal).normalize().to_array());
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }
}

fn update_spline_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Handle<Mesh>, &mut MeshDeformation, &mut Spline<ControlPoint>), Or<(Changed<Spline<ControlPoint>>, Changed<MeshDeformation>)>>
){
    for (handle, mut deform, mut spline) in query.iter_mut() {
        if deform.vertices.is_empty() {
            let original_handle = deform.handle.get_or_insert(handle.clone());
            let Some(mesh) = meshes.get(&original_handle) else { continue };
            deform.cache_mesh(mesh);
        }
        if spline.is_changed() { spline.recalculate_length(); }
        let Some(mesh) = meshes.get_mut(&handle) else { continue };
        deform.apply(&spline, mesh);
    }
}

pub struct SplinePlugin; impl Plugin for SplinePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_spline_system.in_base_set(CoreSet::PostUpdate));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] pub fn sample_normalized_spline(){
        for length in 2..8 {
            let control_points: Vec<ControlPoint> = (0..length)
            .map(|i|ControlPoint{
                position: Vec3::ZERO.lerp(Vec3::X, i as f32 / (length - 1) as f32),
                variant: ControlVariant::Tension(0.0),
                ..Default::default() })
            .collect();
            let mut spline = Spline::new(control_points, 16);
            spline.recalculate_length();
            assert_eq!(spline.nodes.len(), length);
            assert_eq!(spline.length(), 1.0);
            assert_eq!(spline.normalize(0.0), 0.0);
            assert_eq!(spline.sample(0.0).translation, Vec3::ZERO);
            assert_eq!(spline.sample(length as f32 - 1.0).translation, Vec3::X);
            assert_eq!(spline.normalize(1.0), length as f32 - 1.0);
        }
    }
}