use std::collections::HashMap;

use indexmap::IndexSet;
use kdtree::{distance::squared_euclidean, KdTree};

use crate::{
    import::{ImportedFileData, ImportedMesh, ImportedVertex},
    input::CompilationDataInput,
    process::FLOAT_TOLERANCE,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Vector3, Vector4},
    },
};

use super::{
    structures::{
        ProcessedBodyGroupData, ProcessedBodyPartData, ProcessedHardwareBone, ProcessedMeshData, ProcessedMeshVertex, ProcessedModelData, ProcessedStrip,
        ProcessedStripGroup, ProcessedVertexData,
    },
    ProcessingDataError,
};

struct TriangleList {
    material: usize,
    triangles: Vec<[usize; 3]>,
}

impl TriangleList {
    fn new(material: usize, triangles: Vec<[usize; 3]>) -> Self {
        Self { material, triangles }
    }
}

pub fn process_mesh_data(input: &CompilationDataInput, import: &HashMap<String, ImportedFileData>) -> Result<ProcessedModelData, ProcessingDataError> {
    let mut model_data = ProcessedModelData::default();

    for body_group in &input.body_groups {
        let mut body_group_data = ProcessedBodyGroupData::new(body_group.name.clone());

        for body_part in &body_group.parts {
            if body_part.name.len() > 64 {
                todo!("Warn and trim name")
            }

            let mut body_part_data = ProcessedBodyPartData::new(body_part.name.clone());

            let imported_file = import.get(&body_part.model_source).expect("Source File Not Found!");

            let mut material_triangle_lists = create_triangle_lists(&mut model_data, &imported_file.mesh);

            let (unique_vertices, indices_remap) = create_unique_vertices(&imported_file.mesh);

            remap_indices(&mut material_triangle_lists, indices_remap);

            let combined_triangle_list: Vec<[usize; 3]> = material_triangle_lists.iter().flat_map(|list| list.triangles.iter().cloned()).collect();

            let tangents = calculate_vertex_tangents(&unique_vertices, &combined_triangle_list);

            let combined_vertices = combine_vertex_data(&unique_vertices, &tangents, &imported_file);

            for material_triangle_list in material_triangle_lists {
                let mut mesh_data = ProcessedMeshData::new(material_triangle_list.material);
                let mut strip_group_data = ProcessedStripGroup::default();
                let mut strip_data = ProcessedStrip::default();
                let mut bone_changes = IndexSet::new();

                let mut mapped_indices = HashMap::new();
                for triangle in material_triangle_list.triangles {
                    for vertex in triangle {
                        if mapped_indices.contains_key(&vertex) {
                            strip_group_data.indices.push(*mapped_indices.get(&vertex).unwrap());
                            strip_data.indices_count += 1;
                            continue;
                        }

                        strip_group_data.indices.push(strip_group_data.vertices.len());
                        mapped_indices.insert(vertex, strip_group_data.vertices.len());

                        strip_data.indices_count += 1;
                        strip_data.vertex_count += 1;

                        if strip_group_data.vertices.len() > u16::MAX as usize {
                            todo!("Split Mesh Here")
                        }

                        let vertex_data = combined_vertices[vertex].clone();
                        let mut mesh_vertex = ProcessedMeshVertex::default();
                        mesh_vertex.vertex_index = strip_group_data.vertices.len();
                        mesh_vertex.bone_count = vertex_data.bone_count;
                        strip_data.bone_count = if vertex_data.bone_count > strip_data.bone_count {
                            vertex_data.bone_count
                        } else {
                            strip_data.bone_count
                        };
                        for bone_index in 0..vertex_data.bone_count {
                            let bone = vertex_data.bones[bone_index];
                            if bone_changes.contains(&bone) {
                                mesh_vertex.bones[bone_index] = bone_changes.get_index_of(&bone).unwrap();
                                continue;
                            }
                            mesh_vertex.bones[bone_index] = bone;
                            bone_changes.insert(bone);
                        }

                        strip_group_data.vertices.push(mesh_vertex);
                        mesh_data.vertex_data.push(vertex_data);
                    }
                }

                for (hardware_bone, bone_table_bone) in bone_changes.iter().enumerate() {
                    strip_data.hardware_bones.push(ProcessedHardwareBone {
                        hardware_bone,
                        bone_table_bone: *bone_table_bone,
                    })
                }

                strip_group_data.strips.push(strip_data);
                mesh_data.strip_groups.push(strip_group_data);
                body_part_data.meshes.push(mesh_data);
            }

            body_group_data.parts.push(body_part_data);
        }

        model_data.body_groups.push(body_group_data);
    }

    Ok(model_data)
}

fn combine_vertex_data(unique_vertices: &Vec<ImportedVertex>, tangents: &Vec<Vector4>, file_data: &ImportedFileData) -> Vec<ProcessedVertexData> {
    let mut combined_vertices = Vec::with_capacity(unique_vertices.len());

    for (vertex_index, vertex) in unique_vertices.iter().enumerate() {
        let mut combined_vertex = ProcessedVertexData::default();

        create_weight_link(&mut combined_vertex, &vertex.weights, &file_data);

        combined_vertex.position = vertex.position;
        combined_vertex.normal = vertex.normal;
        combined_vertex.uv = vertex.uv;
        combined_vertex.tangent = tangents[vertex_index];

        combined_vertices.push(combined_vertex);
    }

    combined_vertices
}

fn create_weight_link(vertex: &mut ProcessedVertexData, weights: &Vec<(usize, f64)>, file_data: &ImportedFileData) {
    let mut sorted_weights: Vec<(usize, f64)> = weights.iter().cloned().collect();
    sorted_weights.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (weight_index, weight) in sorted_weights.iter().enumerate() {
        if weight_index > 2 {
            log("Vertex had more that 3 weight links! Culling links to 3!", LogLevel::Warn);
            break;
        }

        vertex.weights[weight_index] = weight.1;
        vertex.bones[weight_index] = *file_data.remapped_bones.get(&weight.0).expect("Mapped Bone Not Found!");
        vertex.bone_count = weight_index + 1;
    }

    if vertex.bone_count == 0 {
        todo!("Return Error 0 Vertex Weights");
    }

    normalize_values(&mut vertex.weights, vertex.bone_count);
}

fn normalize_values(arr: &mut [f64; 3], count: usize) {
    if count == 1 {
        arr[0] = 1.0;
        return;
    }

    let magnitude = arr.iter().take(count).map(|&x| x * x).sum::<f64>().sqrt();

    if magnitude < f64::EPSILON {
        todo!("Return Error Of No Weights")
    }

    for i in 0..count {
        arr[i] /= magnitude;
    }
}

fn calculate_vertex_tangents(vertices: &Vec<ImportedVertex>, triangles: &Vec<[usize; 3]>) -> Vec<Vector4> {
    let mut tangents = vec![Vector3::default(); vertices.len()];
    let mut bi_tangents = vec![Vector3::default(); vertices.len()];

    for face in triangles {
        let edge1 = vertices[face[1]].position - vertices[face[0]].position;
        let edge2 = vertices[face[2]].position - vertices[face[0]].position;
        let delta_uv1 = vertices[face[1]].uv - vertices[face[0]].uv;
        let delta_uv2 = vertices[face[2]].uv - vertices[face[0]].uv;

        let area = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        let tangent = Vector3::new(
            area * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            area * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            area * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        )
        .normalize();

        let bi_tangent = Vector3::new(
            area * (delta_uv1.x * edge2.x - delta_uv2.x * edge1.x),
            area * (delta_uv1.x * edge2.y - delta_uv2.x * edge1.y),
            area * (delta_uv1.x * edge2.z - delta_uv2.x * edge1.z),
        )
        .normalize();

        for vertex_index in 0..3 {
            tangents[face[vertex_index]] = tangents[face[vertex_index]] + tangent;
            bi_tangents[face[vertex_index]] = bi_tangents[face[vertex_index]] + bi_tangent;
        }
    }

    let mut calculated_tangents = Vec::with_capacity(vertices.len());
    for index in 0..tangents.len() {
        tangents[index] = tangents[index].normalize();
        bi_tangents[index] = bi_tangents[index].normalize();

        let cross_product = vertices[index].normal.cross(&tangents[index]);
        let sign = if cross_product.dot(&bi_tangents[index]) < 0.0 { -1.0 } else { 1.0 };

        calculated_tangents.push(Vector4::new(tangents[index].x, tangents[index].y, tangents[index].z, sign));
    }

    calculated_tangents
}

fn remap_indices(triangle_lists: &mut Vec<TriangleList>, remap_list: Vec<usize>) {
    for faces in triangle_lists {
        for face in &mut faces.triangles {
            for index in face {
                *index = remap_list[*index];
            }
        }
    }
}

fn create_triangle_lists(model_data: &mut ProcessedModelData, mesh: &ImportedMesh) -> Vec<TriangleList> {
    let mut triangle_lists = Vec::with_capacity(mesh.materials.len());

    for (material, list) in &mesh.materials {
        let material_index = model_data.add_material(material.clone());
        let mut triangles = Vec::new();

        for face in list {
            if face.len() < 3 {
                // Will this happen, shouldn't but just to make sure.
                todo!("Return Error Here")
            }

            if face.len() > 3 {
                todo!("Triangulate Face Here")
            }

            triangles.push([face[0], face[1], face[2]]);
        }

        triangle_lists.push(TriangleList::new(material_index, triangles));
    }

    triangle_lists
}

fn create_unique_vertices(mesh: &ImportedMesh) -> (Vec<ImportedVertex>, Vec<usize>) {
    let mut kd_tree = KdTree::new(3);
    let mut unique_vertices = Vec::new();
    let mut indices_remap = Vec::with_capacity(mesh.vertices.len());
    for vertex in &mesh.vertices {
        let neighbors = kd_tree.within(&vertex.position.as_slice(), FLOAT_TOLERANCE, &squared_euclidean).unwrap();

        if let Some(&(_, index)) = neighbors.iter().find(|(_, &i)| vertex_equals(&vertex, &unique_vertices[i])) {
            indices_remap.push(*index);
            continue;
        }

        kd_tree.add(vertex.position.as_slice(), unique_vertices.len()).unwrap();
        indices_remap.push(unique_vertices.len());
        unique_vertices.push(vertex.clone());
    }

    (unique_vertices, indices_remap)
}

fn vertex_equals(from: &ImportedVertex, to: &ImportedVertex) -> bool {
    if (from.normal.x - to.normal.x).abs() > FLOAT_TOLERANCE
        || (from.normal.y - to.normal.y).abs() > FLOAT_TOLERANCE
        || (from.normal.z - to.normal.z).abs() > FLOAT_TOLERANCE
    {
        return false;
    }

    if (from.uv.x - to.uv.x).abs() > FLOAT_TOLERANCE || (from.uv.y - to.uv.y).abs() > FLOAT_TOLERANCE {
        return false;
    }

    if from.weights.len() != to.weights.len() {
        return false;
    }

    if from.weights.iter().zip(to.weights.iter()).any(|(from_link, to_link)| {
        if from_link.0 != to_link.0 {
            return true;
        }

        (from_link.1 - to_link.1).abs() > FLOAT_TOLERANCE
    }) {
        return false;
    }

    true
}
