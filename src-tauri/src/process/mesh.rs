use std::{collections::HashMap, sync::Arc};

use indexmap::IndexSet;
use kdtree::{distance::squared_euclidean, KdTree};
use tauri::State;
use thiserror::Error as ThisError;

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
    ProcessedStripGroup, ProcessedVertex, FLOAT_TOLERANCE,
};

#[derive(Debug, Default)]
struct CombinedMesh {
    vertices: Vec<ImportVertex>,
    polygons: Vec<CombinedMeshFace>,
}

#[derive(Debug, Default)]
struct CombinedMeshFace {
    material: usize,
    vertex_offset: usize,
    faces: Vec<Vec<usize>>,
}

#[derive(Debug, Default)]
struct TriangleList {
    material: usize,
    vertex_offset: usize,
    triangles: Vec<[usize; 3]>,
}

#[derive(Debug, ThisError)]
pub enum ProcessingMeshError {
    #[error("Part Not Found: {0}")]
    PartNotFound(String),
    #[error("Face Has Less Than 3 Vertices")]
    IncompleteFace,
}

pub fn process_mesh_data(
    input: &ImputedCompilationData,
    import: &State<FileManager>,
    bone_table: &BoneTable,
) -> Result<ProcessedModelData, ProcessingMeshError> {
    let mut processed_model_data = ProcessedModelData::default();

    for imputed_body_part in &input.body_parts {
        let mut processed_body_part = ProcessedBodyPart {
            name: imputed_body_part.name.clone(),
            ..Default::default()
        };

        for imputed_model in &imputed_body_part.models {
            let mut processed_model = ProcessedModel {
                name: imputed_model.name.clone(),
                ..Default::default()
            };

            if processed_model.name.len() > 64 {
                log("Model Part Name Longer That 64! Trimming!", LogLevel::Warn);
                processed_model.name.truncate(64);
            }

            let imported_file = import.get_file(&imputed_model.model_source).expect("Source File Not Found!");

            let combined_meshes = create_combined_meshes(imputed_model, &imported_file, &mut processed_model_data)?;

            let mut material_triangle_lists = create_triangle_lists(&combined_meshes)?;

            let (unique_vertices, indices_remap) = create_unique_vertices(&combined_meshes);

            remap_indices(&mut material_triangle_lists, indices_remap);

            let combined_triangle_list: Vec<[usize; 3]> = material_triangle_lists.iter().flat_map(|list| list.triangles.iter().cloned()).collect();

            let tangents = calculate_vertex_tangents(&unique_vertices, &combined_triangle_list);

            let mapped_bone = bone_table.remapped_bones.get(&imputed_model.model_source).expect("Source File Not Remapped!");

            let combined_vertices = combine_vertex_data(&unique_vertices, &tangents, mapped_bone);

            for material_triangle_list in material_triangle_lists {
                let mut processed_mesh = ProcessedMesh {
                    material: material_triangle_list.material,
                    ..Default::default()
                };
                let mut processed_strip_group = ProcessedStripGroup::default();
                let mut processed_strip = ProcessedStrip::default();

                let mut mapped_indices = HashMap::new();
                let mut hardware_bones = IndexSet::new();
                for triangle in material_triangle_list.triangles {
                    let new_vertex_count = triangle.iter().filter(|&&value| mapped_indices.contains_key(&value)).count();
                    if processed_strip_group.vertices.len() + new_vertex_count > (u16::MAX - 3) as usize {
                        for hardware_bone in &hardware_bones {
                            let processed_hardware_bone = ProcessedHardwareBone {
                                hardware_bone: processed_strip.hardware_bones.len(),
                                bone_table_bone: *hardware_bone,
                            };

                            processed_strip.hardware_bones.push(processed_hardware_bone);
                        }

                        processed_strip.vertex_count = processed_strip_group.vertices.len();
                        processed_strip.indices_count = processed_strip_group.indices.len();
                        processed_strip_group.strips.push(processed_strip);
                        processed_mesh.strip_groups.push(processed_strip_group);
                        processed_model.meshes.push(processed_mesh);

                        mapped_indices.clear();
                        hardware_bones.clear();

                        processed_mesh = ProcessedMesh::default();
                        processed_mesh.material = material_triangle_list.material;
                        processed_strip_group = ProcessedStripGroup::default();
                        processed_strip = ProcessedStrip::default();
                    }

                    for index in triangle {
                        if mapped_indices.contains_key(&index) {
                            processed_strip_group.indices.push(*mapped_indices.get(&index).unwrap());
                            continue;
                        }

                        assert!(processed_strip_group.vertices.len() < u16::MAX as usize, "Too Many Vertices!");

                        processed_strip_group.indices.push(processed_strip_group.vertices.len() as u16);
                        mapped_indices.insert(index, processed_strip_group.vertices.len() as u16);

                        let vertex_data = combined_vertices[index].clone();
                        let mut processed_mesh_vertex = ProcessedMeshVertex {
                            vertex_index: processed_strip_group.vertices.len(),
                            bone_count: vertex_data.bone_count,
                            ..Default::default()
                        };
                        processed_strip.bone_count = if vertex_data.bone_count > processed_strip.bone_count {
                            vertex_data.bone_count
                        } else {
                            processed_strip.bone_count
                        };

                        for bone_index in 0..vertex_data.bone_count {
                            let bone = vertex_data.bones[bone_index];

                            let hardware_bone_index = hardware_bones.insert_full(bone).0;

                            processed_mesh_vertex.bones[bone_index] = hardware_bone_index;
                        }

                        processed_strip_group.vertices.push(processed_mesh_vertex);
                        processed_mesh.vertex_data.push(vertex_data);
                    }
                }

                for hardware_bone in hardware_bones {
                    let processed_hardware_bone = ProcessedHardwareBone {
                        hardware_bone: processed_strip.hardware_bones.len(),
                        bone_table_bone: hardware_bone,
                    };

                    processed_strip.hardware_bones.push(processed_hardware_bone);
                }

                processed_strip.vertex_count = processed_strip_group.vertices.len();
                processed_strip.indices_count = processed_strip_group.indices.len();
                processed_strip_group.strips.push(processed_strip);
                processed_mesh.strip_groups.push(processed_strip_group);
                processed_model.meshes.push(processed_mesh);
            }

            log(format!("Processed {} vertices", combined_vertices.len()), LogLevel::Verbose);

            processed_body_part.parts.push(processed_model);
        }

        processed_model_data.body_parts.push(processed_body_part);
    }

    Ok(processed_model_data)
}

fn create_combined_meshes(
    imputed_model: &ImputedModel,
    imported_file: &Arc<ImportFileData>,
    processed_model_data: &mut ProcessedModelData,
) -> Result<CombinedMesh, ProcessingMeshError> {
    let mut combined_mesh = CombinedMesh::default();

    // FIXME: This is a temporary solution to support single part models.
    if imported_file.parts.len() == 1 {
        let part = imported_file.parts.first().unwrap();

        combined_mesh.vertices.extend(part.vertices.iter().cloned());

        for (material, faces) in &part.polygons {
            let material_index = processed_model_data.materials.insert_full(material.clone()).0;

            let combined_mesh_face = CombinedMeshFace {
                material: material_index,
                faces: faces.clone(),
                ..Default::default()
            };

            combined_mesh.polygons.push(combined_mesh_face);
        }

        return Ok(combined_mesh);
    }

    for imputed_part_name in &imputed_model.part_name {
        let import_part = match imported_file.parts.iter().find(|part| part.name == *imputed_part_name) {
            Some(part) => part,
            None => return Err(ProcessingMeshError::PartNotFound(imputed_part_name.clone())),
        };

        for (material, faces) in &import_part.polygons {
            let material_index = processed_model_data.materials.insert_full(material.clone()).0;

            let combined_mesh_face = CombinedMeshFace {
                material: material_index,
                vertex_offset: combined_mesh.vertices.len(),
                faces: faces.clone(),
            };

            combined_mesh.polygons.push(combined_mesh_face);
        }

        combined_mesh.vertices.extend(import_part.vertices.iter().cloned());
    }

    Ok(combined_mesh)
}

fn create_triangle_lists(combined_meshes: &CombinedMesh) -> Result<Vec<TriangleList>, ProcessingMeshError> {
    let mut triangle_lists = Vec::new();

    for mesh in &combined_meshes.polygons {
        let mut triangle_list = TriangleList {
            material: mesh.material,
            vertex_offset: mesh.vertex_offset,
            ..Default::default()
        };

        for face in &mesh.faces {
            if face.len() < 3 {
                return Err(ProcessingMeshError::IncompleteFace);
            }

            if face.len() > 3 {
                let triangulated_faces = triangulate_face(face, &combined_meshes.vertices, mesh.vertex_offset);
                triangle_list.triangles.extend(triangulated_faces);
                continue;
            }

            triangle_list.triangles.push([face[2], face[1], face[0]]);
        }

        triangle_lists.push(triangle_list);
    }

    Ok(triangle_lists)
}

fn triangulate_face(_face: &[usize], _vertices: &[ImportVertex], _vertex_offset: usize) -> Vec<[usize; 3]> {
    todo!("Triangulate Face Here")
}

fn create_unique_vertices(mesh: &CombinedMesh) -> (Vec<ImportVertex>, Vec<usize>) {
    let mut kd_tree = KdTree::new(3);
    let mut unique_vertices = Vec::new();
    let mut indices_remap = Vec::with_capacity(mesh.vertices.len());
    for vertex in &mesh.vertices {
        let neighbors = kd_tree.within(&vertex.position.as_slice(), FLOAT_TOLERANCE, &squared_euclidean).unwrap();

        if let Some(&(_, index)) = neighbors.iter().find(|(_, &i)| vertex_equals(vertex, &unique_vertices[i])) {
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

fn remap_indices(triangle_lists: &mut Vec<TriangleList>, remap_list: Vec<usize>) {
    for faces in triangle_lists {
        for face in &mut faces.triangles {
            for index in face {
                *index = remap_list[faces.vertex_offset + *index];
            }
        }
    }
}

fn calculate_vertex_tangents(vertices: &[ImportVertex], triangles: &Vec<[usize; 3]>) -> Vec<Vector4> {
    let mut tangents = vec![Vector3::default(); vertices.len()];
    let mut bi_tangents = vec![Vector3::default(); vertices.len()];

    for face in triangles {
        let edge1 = vertices[face[1]].position - vertices[face[0]].position;
        let edge2 = vertices[face[2]].position - vertices[face[0]].position;
        let delta_uv1 = vertices[face[1]].texture_coordinate - vertices[face[0]].texture_coordinate;
        let delta_uv2 = vertices[face[2]].texture_coordinate - vertices[face[0]].texture_coordinate;

        let denominator = delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y;

        if denominator.abs() < f64::EPSILON {
            continue;
        }

        let area = 1.0 / denominator;

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

fn combine_vertex_data(unique_vertices: &[ImportVertex], tangents: &[Vector4], mapped_bone: &HashMap<usize, usize>) -> Vec<ProcessedVertex> {
    let mut combined_vertices = Vec::with_capacity(unique_vertices.len());

    for (vertex_index, vertex) in unique_vertices.iter().enumerate() {
        let mut combined_vertex = ProcessedVertex::default();

        create_weight_link(&mut combined_vertex, &vertex.links, mapped_bone);

        combined_vertex.position = vertex.position;
        combined_vertex.normal = vertex.normal.normalize();
        combined_vertex.uv = vertex.texture_coordinate;
        combined_vertex.tangent = tangents[vertex_index];

        combined_vertices.push(combined_vertex);
    }

    combined_vertices
}

fn create_weight_link(vertex: &mut ProcessedVertex, weights: &[ImportLink], mapped_bone: &HashMap<usize, usize>) {
    let mut sorted_weights: Vec<ImportLink> = weights.to_vec();
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
    let sum = arr.iter().take(count).sum::<f64>();

    if sum < f64::EPSILON {
        todo!("Return Error Of No Weights")
    }

    for weight in arr.iter_mut().take(count) {
        *weight /= sum;
    }

    for weight in arr.iter_mut().skip(count) {
        *weight = 0.0;
    }
}
