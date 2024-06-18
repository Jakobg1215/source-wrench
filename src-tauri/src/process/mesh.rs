use std::{collections::HashMap, sync::Arc};

use indexmap::IndexSet;
use kdtree::{distance::squared_euclidean, KdTree};
use tauri::State;

use crate::{
    import::{FileManager, ImportFileData, ImportLink, ImportVertex},
    input::{ImputedCompilationData, ImputedModel},
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Vector3, Vector4},
    },
};

use super::{
    bones::BoneTable, ProcessedBodyPart, ProcessedHardwareBone, ProcessedMesh, ProcessedMeshVertex, ProcessedModel, ProcessedModelData, ProcessedStrip,
    ProcessedStripGroup, ProcessedVertex, ProcessingDataError, FLOAT_TOLERANCE,
};

// FIXME: So much cloning, need to find a way to avoid it.

struct TriangleList {
    material: usize,
    triangles: Vec<[usize; 3]>,
}

impl TriangleList {
    fn new(material: usize, triangles: Vec<[usize; 3]>) -> Self {
        Self { material, triangles }
    }
}

#[derive(Default, Debug)]
struct CombinedMesh {
    vertices: Vec<ImportVertex>,
    polygons: HashMap<String, Vec<Vec<usize>>>,
    // flexes: Vec<ImportFlex>,
}

pub fn process_mesh_data(
    input: &ImputedCompilationData,
    import: &State<FileManager>,
    bone_table: &BoneTable,
) -> Result<ProcessedModelData, ProcessingDataError> {
    let mut processed_model_data = ProcessedModelData::default();

    for body_part in &input.body_parts {
        let mut body_part_data = ProcessedBodyPart::default();
        body_part_data.name = body_part.name.clone();

        for model in &body_part.models {
            if model.name.len() > 64 {
                todo!("Warn and trim name")
            }

            let mut model_data = ProcessedModel::default();
            model_data.name = model.name.clone();

            let imported_file = import.get_file(&model.model_source).expect("Source File Not Found!");

            let combined_mesh = create_combined_mesh(&imported_file, &model);

            let mut material_triangle_lists = create_triangle_lists(&mut processed_model_data, &combined_mesh);

            let (unique_vertices, indices_remap) = create_unique_vertices(&combined_mesh);

            remap_indices(&mut material_triangle_lists, indices_remap);

            let combined_triangle_list: Vec<[usize; 3]> = material_triangle_lists.iter().flat_map(|list| list.triangles.iter().cloned()).collect();

            let tangents = calculate_vertex_tangents(&unique_vertices, &combined_triangle_list);

            let mapped_bone = bone_table.remapped_bones.get(&model.model_source).expect("Source File Not Remapped!");

            let combined_vertices = combine_vertex_data(&unique_vertices, &tangents, &mapped_bone);

            for material_triangle_list in material_triangle_lists {
                let mut mesh_data = ProcessedMesh::default();
                mesh_data.material = material_triangle_list.material;
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
                model_data.meshes.push(mesh_data);
            }

            body_part_data.parts.push(model_data);
        }

        processed_model_data.body_parts.push(body_part_data);
    }

    Ok(processed_model_data)
}

fn create_combined_mesh(imported_file: &Arc<ImportFileData>, body_part: &ImputedModel) -> CombinedMesh {
    if imported_file.parts.len() == 0 {
        todo!("Return Error Here")
    }

    if imported_file.parts.len() == 1 || body_part.part_name.len() == 1 {
        let mut combined = CombinedMesh::default();

        let part = imported_file.parts.first().unwrap();
        combined.vertices = part.vertices.clone();
        combined.polygons = part.polygons.clone();

        return combined;
    }

    todo!("Support Multiple Parts Here")
}

fn combine_vertex_data(unique_vertices: &Vec<ImportVertex>, tangents: &Vec<Vector4>, mapped_bone: &HashMap<usize, usize>) -> Vec<ProcessedVertex> {
    let mut combined_vertices = Vec::with_capacity(unique_vertices.len());

    for (vertex_index, vertex) in unique_vertices.iter().enumerate() {
        let mut combined_vertex = ProcessedVertex::default();

        create_weight_link(&mut combined_vertex, &vertex.links, &mapped_bone);

        combined_vertex.position = vertex.position;
        combined_vertex.normal = vertex.normal;
        combined_vertex.uv = vertex.texture_coordinate;
        combined_vertex.tangent = tangents[vertex_index];

        combined_vertices.push(combined_vertex);
    }

    combined_vertices
}

fn create_weight_link(vertex: &mut ProcessedVertex, weights: &Vec<ImportLink>, mapped_bone: &HashMap<usize, usize>) {
    let mut sorted_weights: Vec<ImportLink> = weights.iter().cloned().collect();
    sorted_weights.sort_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap());

    for (weight_index, weight) in sorted_weights.iter().enumerate() {
        if weight_index > 2 {
            log("Vertex had more that 3 weight links! Culling links to 3!", LogLevel::Warn);
            break;
        }

        vertex.weights[weight_index] = weight.weight;
        vertex.bones[weight_index] = *mapped_bone.get(&weight.bone).expect("Mapped Bone Not Found!");
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

fn calculate_vertex_tangents(vertices: &Vec<ImportVertex>, triangles: &Vec<[usize; 3]>) -> Vec<Vector4> {
    let mut tangents = vec![Vector3::default(); vertices.len()];
    let mut bi_tangents = vec![Vector3::default(); vertices.len()];

    for face in triangles {
        let edge1 = vertices[face[1]].position - vertices[face[0]].position;
        let edge2 = vertices[face[2]].position - vertices[face[0]].position;
        let delta_uv1 = vertices[face[1]].texture_coordinate - vertices[face[0]].texture_coordinate;
        let delta_uv2 = vertices[face[2]].texture_coordinate - vertices[face[0]].texture_coordinate;

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

fn create_triangle_lists(model_data: &mut ProcessedModelData, mesh: &CombinedMesh) -> Vec<TriangleList> {
    let mut triangle_lists = Vec::with_capacity(mesh.polygons.len());

    for (material, list) in &mesh.polygons {
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

fn create_unique_vertices(mesh: &CombinedMesh) -> (Vec<ImportVertex>, Vec<usize>) {
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

fn vertex_equals(from: &ImportVertex, to: &ImportVertex) -> bool {
    if (from.normal.x - to.normal.x).abs() > FLOAT_TOLERANCE
        || (from.normal.y - to.normal.y).abs() > FLOAT_TOLERANCE
        || (from.normal.z - to.normal.z).abs() > FLOAT_TOLERANCE
    {
        return false;
    }

    if (from.texture_coordinate.x - to.texture_coordinate.x).abs() > FLOAT_TOLERANCE
        || (from.texture_coordinate.y - to.texture_coordinate.y).abs() > FLOAT_TOLERANCE
    {
        return false;
    }

    if from.links.len() != to.links.len() {
        return false;
    }

    if from.links.iter().zip(to.links.iter()).any(|(from_link, to_link)| {
        if from_link.bone != to_link.bone {
            return true;
        }

        (from_link.weight - to_link.weight).abs() > FLOAT_TOLERANCE
    }) {
        return false;
    }

    true
}
