use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::mesh::{Indices, Mesh, VertexAttributeValues};

pub fn merge_meshes(
    meshes: &[Mesh],
    transforms: Option<&[Transform]>,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut tangents: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    let mut indices: Vec<u32> = Vec::new();
    let mut index_offset: u32 = 0;

    for (i, mesh) in meshes.iter().enumerate() {
        let Some(VertexAttributeValues::Float32x3(mesh_positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { continue };        
        if let Some(transform) = transforms.map(|list|list[i].compute_matrix()) {
            for &position in mesh_positions {
                positions.push(transform.transform_point3(position.into()).into());
            }
            if let Some(VertexAttributeValues::Float32x3(mesh_normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
                let inverse_transpose = Mat3::from_mat4(transform.inverse().transpose());
                for &normal in mesh_normals {
                    normals.push(inverse_transpose.mul_vec3(normal.into()).normalize_or_zero().into());
                }
            }
        } else {
            positions.extend_from_slice(&mesh_positions);

            if let Some(VertexAttributeValues::Float32x3(mesh_normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
                normals.extend_from_slice(&mesh_normals);
            }
        }

        if let Some(VertexAttributeValues::Float32x4(mesh_tangents)) = mesh.attribute(Mesh::ATTRIBUTE_TANGENT) {
            tangents.extend_from_slice(&mesh_tangents);
        }
        if let Some(VertexAttributeValues::Float32x2(mesh_uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            uvs.extend_from_slice(&mesh_uvs);
        }
        if let Some(VertexAttributeValues::Float32x4(mesh_colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            colors.extend_from_slice(&mesh_colors);
        }

        if let Some(Indices::U32(mesh_indices)) = mesh.indices() {
            for index in mesh_indices {
                indices.push(*index + index_offset);
            }
        }
        index_offset += mesh_positions.len() as u32;
    }

    let mut mesh: Mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    if !normals.is_empty() { mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals); }
    if !tangents.is_empty() { mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents); }
    if !uvs.is_empty() { mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs); }
    if !colors.is_empty() { mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors); }
    if !indices.is_empty() { mesh.set_indices(Some(Indices::U32(indices))); }
    mesh
}

pub fn assign_mesh_color(mesh: &mut Mesh, color: Color){
    if let Some(VertexAttributeValues::Float32x4(ref mut colors)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        colors.fill(color.into());
    } else {
        let vertex_count: usize = mesh.count_vertices();
        let colors: Vec<[f32; 4]> = vec![color.into(); vertex_count];
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
}