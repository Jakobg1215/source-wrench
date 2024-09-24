use std::{collections::HashMap, usize};

use indexmap::{IndexMap, IndexSet};
use kdtree::{distance::squared_euclidean, KdTree};
use tauri::State;
use thiserror::Error as ThisError;

use crate::{
    import::{FileManager, ImportLink, ImportPart, ImportVertex},
    input::ImputedCompilationData,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Vector3, Vector4},
    },
};

use super::{
    ProcessedBodyPart, ProcessedBoneData, ProcessedHardwareBone, ProcessedMesh, ProcessedMeshVertex, ProcessedModel, ProcessedModelData, ProcessedStrip,
    ProcessedStripGroup, ProcessedVertex, FLOAT_TOLERANCE, MAX_HARDWARE_BONES_PER_STRIP, VERTEX_CACHE_SIZE,
};

#[derive(Debug, Default)]
struct TriangleList {
    vertices: Vec<ProcessedVertex>,
    triangles: Vec<[usize; 3]>,
}

#[derive(Debug, ThisError)]
pub enum ProcessingMeshError {
    #[error("Model File Source Not Loaded")]
    FileSourceNotLoaded,
    #[error("Part Not Found: {0}")]
    PartNotFound(String),
    #[error("Face Was Incomplete")]
    IncompleteFace,
    #[error("Vertex Weights Were Not Found")]
    VertexHasNoWeights,
}

pub fn process_mesh_data(
    input: &ImputedCompilationData,
    import: &State<FileManager>,
    bone_table: &ProcessedBoneData,
) -> Result<ProcessedModelData, ProcessingMeshError> {
    let mut processed_model_data = ProcessedModelData::default();

    for imputed_body_part in &input.body_parts {
        let mut processed_body_part = ProcessedBodyPart {
            name: imputed_body_part.name.clone(),
            ..Default::default()
        };

        for imputed_model in &imputed_body_part.models {
            if imputed_model.is_blank {
                processed_body_part.parts.push(ProcessedModel::default());
                continue;
            }

            let mut processed_model = ProcessedModel {
                name: imputed_model.name.clone(),
                ..Default::default()
            };

            if processed_model.name.len() > 64 {
                log("Model Part Name Longer That 64! Trimming!", LogLevel::Warn);
                processed_model.name.truncate(64);
            }

            let imported_file = match import.get_file(&imputed_model.file_source) {
                Some(file) => file,
                None => {
                    return Err(ProcessingMeshError::FileSourceNotLoaded);
                }
            };

            let mut triangle_lists = create_triangle_lists(
                &imputed_model.part_names,
                &imported_file.parts,
                &mut processed_model_data.materials,
                bone_table.remapped_bones.get(&imputed_model.file_source).unwrap(),
            )?;

            if triangle_lists.is_empty() {
                log("Model Had No Parts! Defaulting To Blank!", LogLevel::Warn);
                processed_body_part.parts.push(ProcessedModel::default());
                continue;
            }

            for list in triangle_lists.values_mut() {
                calculate_vertex_tangents(list);
                optimize_indices_order(list);
            }

            // Sort Triangles for optimization

            // TODO: Split this into functions as its a bit confusing.
            let mut vertex_count = 0;
            let mut triangle_count = 0;
            for (material, list) in triangle_lists {
                let mut processed_mesh = ProcessedMesh {
                    material,
                    ..Default::default()
                };
                vertex_count += list.vertices.len();
                let mut processed_strip_group = ProcessedStripGroup::default();
                let mut processed_strip = ProcessedStrip::default();

                let mut mapped_indices = HashMap::new();
                let mut hardware_bones = IndexSet::new();
                for triangle in list.triangles {
                    triangle_count += 1;
                    let new_vertex_count = triangle.iter().filter(|&&value| mapped_indices.contains_key(&value)).count();
                    if processed_strip_group.vertices.len() + new_vertex_count > u16::MAX as usize {
                        processed_strip_group.strips.push(processed_strip);
                        processed_mesh.strip_groups.push(processed_strip_group);
                        processed_model.meshes.push(processed_mesh);

                        mapped_indices.clear();
                        hardware_bones.clear();

                        processed_mesh = ProcessedMesh {
                            material,
                            ..Default::default()
                        };
                        processed_strip_group = ProcessedStripGroup::default();
                        processed_strip = ProcessedStrip::default();
                    }

                    let new_hardware_bone_count = triangle
                        .iter()
                        .map(|&vertex_index| list.vertices[vertex_index])
                        .collect::<Vec<_>>()
                        .iter()
                        .map(|vertex| {
                            vertex
                                .bones
                                .iter()
                                .take(vertex.bone_count)
                                .filter(|&bone| hardware_bones.contains(bone))
                                .count()
                        })
                        .sum::<usize>();

                    if hardware_bones.len() + new_hardware_bone_count > MAX_HARDWARE_BONES_PER_STRIP {
                        let new_processed_strip = ProcessedStrip {
                            indices_offset: processed_strip.indices_offset + processed_strip.indices_count,
                            vertex_offset: processed_strip.vertex_offset + processed_strip.vertex_count,
                            ..Default::default()
                        };

                        hardware_bones.clear();
                        processed_strip_group.strips.push(processed_strip);
                        processed_strip = new_processed_strip;
                    }

                    for index in triangle {
                        if mapped_indices.contains_key(&index) {
                            processed_strip_group.indices.push(*mapped_indices.get(&index).unwrap());
                            processed_strip.indices_count += 1;
                            continue;
                        }

                        let vertex_data = list.vertices[index];
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

                            let (hardware_bone_index, new_hardware_bone) = hardware_bones.insert_full(bone);

                            processed_mesh_vertex.bones[bone_index] = hardware_bone_index;
                            if new_hardware_bone {
                                let processed_hardware_bone = ProcessedHardwareBone {
                                    hardware_bone: hardware_bone_index,
                                    bone_table_bone: bone,
                                };
                                processed_strip.hardware_bones.push(processed_hardware_bone);
                            }
                        }

                        processed_strip_group.indices.push(processed_strip_group.vertices.len() as u16);
                        mapped_indices.insert(index, processed_strip_group.vertices.len() as u16);
                        processed_strip.indices_count += 1;

                        processed_strip_group.vertices.push(processed_mesh_vertex);
                        processed_mesh.vertex_data.push(vertex_data);
                        processed_strip.vertex_count += 1;
                    }
                }

                processed_strip_group.strips.push(processed_strip);
                processed_mesh.strip_groups.push(processed_strip_group);
                processed_model.meshes.push(processed_mesh);
            }
            log(
                format!("Processed {} triangles and {} vertices", triangle_count, vertex_count),
                LogLevel::Verbose,
            );
            processed_body_part.parts.push(processed_model);
        }

        processed_model_data.body_parts.push(processed_body_part);
    }

    Ok(processed_model_data)
}

fn create_triangle_lists(
    part_names: &[String],
    parts: &[ImportPart],
    material_table: &mut IndexSet<String>,
    mapped_bone: &[usize],
) -> Result<IndexMap<usize, TriangleList>, ProcessingMeshError> {
    let mut triangle_lists = IndexMap::new();
    let mut processed_vertices_trees = IndexMap::new();

    for imputed_part_name in part_names {
        let import_part = match parts.iter().find(|part| part.name == *imputed_part_name) {
            Some(part) => part,
            None => return Err(ProcessingMeshError::PartNotFound(imputed_part_name.clone())),
        };

        for (material, faces) in &import_part.polygons {
            let material_index = material_table.insert_full(material.clone()).0;

            let triangle_list: &mut TriangleList = triangle_lists.entry(material_index).or_default();
            let processed_vertices_tree: &mut KdTree<f64, usize, [f64; 3]> = processed_vertices_trees.entry(material_index).or_insert_with(|| KdTree::new(3));

            for face in faces {
                if face.len() < 3 {
                    return Err(ProcessingMeshError::IncompleteFace);
                }

                let triangulated_face = triangulate_face(face, &import_part.vertices);

                for mut triangle in triangulated_face {
                    for point in &mut triangle {
                        let import_vertex = &import_part.vertices[*point];

                        let mut processed_vertex = ProcessedVertex {
                            position: import_vertex.position,
                            normal: import_vertex.normal.normalize(),
                            texture_coordinate: import_vertex.texture_coordinate,
                            ..Default::default()
                        };

                        // TODO: StudioMDL will move the vertices with the define bones to fix up the bind pose, this should be done here.

                        create_bone_links(&mut processed_vertex, import_vertex, mapped_bone)?;

                        let neighbors = processed_vertices_tree
                            .within(&processed_vertex.position.as_slice(), FLOAT_TOLERANCE, &squared_euclidean)
                            .unwrap();

                        if let Some(&(_, index)) = neighbors.iter().find(|(_, &i)| vertex_equals(&processed_vertex, &triangle_list.vertices[i])) {
                            *point = *index;
                            continue;
                        }

                        processed_vertices_tree
                            .add(processed_vertex.position.as_slice(), triangle_list.vertices.len())
                            .unwrap();
                        *point = triangle_list.vertices.len();
                        triangle_list.vertices.push(processed_vertex);
                    }

                    triangle_list.triangles.push([triangle[0], triangle[1], triangle[2]]);
                }
            }
        }
    }

    Ok(triangle_lists)
}

fn triangulate_face(face: &[usize], _vertices: &[ImportVertex]) -> Vec<[usize; 3]> {
    if face.len() == 3 {
        return vec![[face[2], face[1], face[0]]];
    }
    todo!("Triangulate Face Here")
}

fn create_bone_links(processed_vertex: &mut ProcessedVertex, vertex: &ImportVertex, mapped_bone: &[usize]) -> Result<(), ProcessingMeshError> {
    let remapped_weights = vertex
        .links
        .iter()
        .map(|link| ImportLink {
            bone: mapped_bone[link.bone],
            weight: link.weight,
        })
        .collect::<Vec<_>>();

    for weight in remapped_weights {
        if processed_vertex.bone_count == 3 {
            log("Vertex had more that 3 weight links! Culling links to 3!", LogLevel::Warn);
            break;
        }

        processed_vertex.weights[processed_vertex.bone_count] = weight.weight;
        processed_vertex.bones[processed_vertex.bone_count] = weight.bone;
        processed_vertex.bone_count += 1;
    }

    if processed_vertex.bone_count == 0 {
        // This is technically a failure of the compiler if the imports are not added a bone.
        return Err(ProcessingMeshError::VertexHasNoWeights);
    }

    normalize_bone_links(&mut processed_vertex.weights, processed_vertex.bone_count)
}

fn normalize_bone_links(arr: &mut [f64; 3], count: usize) -> Result<(), ProcessingMeshError> {
    let sum = arr.iter().take(count).sum::<f64>();

    if sum < f64::EPSILON {
        return Err(ProcessingMeshError::VertexHasNoWeights);
    }

    for weight in arr.iter_mut().take(count) {
        *weight /= sum;
    }

    for weight in arr.iter_mut().skip(count) {
        *weight = 0.0;
    }

    Ok(())
}

fn vertex_equals(from: &ProcessedVertex, to: &ProcessedVertex) -> bool {
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

    if from.bone_count != to.bone_count {
        return false;
    }

    if from.bones.iter().zip(to.bones.iter()).any(|(from_bone, to_bone)| from_bone != to_bone) {
        return false;
    }

    if from
        .weights
        .iter()
        .zip(to.weights.iter())
        .any(|(from_weight, to_weight)| (from_weight - to_weight).abs() > FLOAT_TOLERANCE)
    {
        return false;
    }

    true
}

fn calculate_vertex_tangents(triangle_list: &mut TriangleList) {
    let mut tangents = vec![Vector3::default(); triangle_list.vertices.len()];
    let mut bi_tangents = vec![Vector3::default(); triangle_list.vertices.len()];

    for face in &triangle_list.triangles {
        let edge1 = triangle_list.vertices[face[1]].position - triangle_list.vertices[face[0]].position;
        let edge2 = triangle_list.vertices[face[2]].position - triangle_list.vertices[face[0]].position;
        let delta_uv1 = triangle_list.vertices[face[1]].texture_coordinate - triangle_list.vertices[face[0]].texture_coordinate;
        let delta_uv2 = triangle_list.vertices[face[2]].texture_coordinate - triangle_list.vertices[face[0]].texture_coordinate;

        let denominator = delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y;

        if denominator.abs() < f64::EPSILON {
            for vertex_index in 0..3 {
                tangents[face[vertex_index]] = Vector3::new(1.0, 0.0, 0.0);
                bi_tangents[face[vertex_index]] = Vector3::new(0.0, 1.0, 0.0);
            }
            continue;
        }

        let area = 1.0 / denominator;

        let tangent = Vector3::new(
            area * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            area * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            area * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        );

        let bi_tangent = Vector3::new(
            area * (delta_uv1.x * edge2.x - delta_uv2.x * edge1.x),
            area * (delta_uv1.x * edge2.y - delta_uv2.x * edge1.y),
            area * (delta_uv1.x * edge2.z - delta_uv2.x * edge1.z),
        );

        for vertex_index in 0..3 {
            tangents[face[vertex_index]] = tangents[face[vertex_index]] + tangent;
            tangents[face[vertex_index]] = tangents[face[vertex_index]].normalize();
            bi_tangents[face[vertex_index]] = bi_tangents[face[vertex_index]] + bi_tangent;
            bi_tangents[face[vertex_index]] = bi_tangents[face[vertex_index]].normalize();
        }
    }

    for index in 0..tangents.len() {
        let cross_product = triangle_list.vertices[index].normal.cross(tangents[index]);
        let sign = if cross_product.dot(bi_tangents[index]) < 0.0 { -1.0 } else { 1.0 };

        triangle_list.vertices[index].tangent = Vector4::new(tangents[index].x, tangents[index].y, tangents[index].z, sign);
    }
}

// Took from https://github.com/zeux/meshoptimizer/blob/master/src/vcacheoptimizer.cpp
fn optimize_indices_order(list: &mut TriangleList) {
    let indices = list.triangles.iter().flatten().copied().collect::<Vec<_>>();
    let mut adjacency = build_triangle_adjacency(list.vertices.len(), &indices);

    let mut vertex_scores = vec![0.0; list.vertices.len()];
    for (vertex_index, vertex_score) in vertex_scores.iter_mut().enumerate() {
        *vertex_score = calculate_vertex_scores(None, adjacency.counts[vertex_index]);
    }

    let mut triangle_scores = vec![0.0; list.triangles.len()];
    for (triangle_index, triangle_score) in triangle_scores.iter_mut().enumerate() {
        let point1 = indices[triangle_index * 3];
        let point2 = indices[triangle_index * 3 + 1];
        let point3 = indices[triangle_index * 3 + 2];
        *triangle_score = vertex_scores[point1] + vertex_scores[point2] + vertex_scores[point3];
    }

    let mut emitted_flags = vec![false; list.triangles.len()];

    let mut cache = [0; VERTEX_CACHE_SIZE + 4];
    let mut cache_new = [0; VERTEX_CACHE_SIZE + 4];
    let mut cache_count = 0;

    let mut current_triangle = 0;
    let mut input_cursor = 1;
    let mut output_triangle = 0;

    while current_triangle != usize::MAX {
        let point1 = indices[current_triangle * 3];
        let point2 = indices[current_triangle * 3 + 1];
        let point3 = indices[current_triangle * 3 + 2];

        list.triangles[output_triangle] = [point1, point2, point3];
        output_triangle += 1;

        emitted_flags[current_triangle] = true;
        triangle_scores[current_triangle] = 0.0;

        let mut cache_write = 0;
        cache_new[cache_write] = point1;
        cache_write += 1;
        cache_new[cache_write] = point2;
        cache_write += 1;
        cache_new[cache_write] = point3;
        cache_write += 1;

        for index in cache.iter().take(cache_count) {
            cache_new[cache_write] = *index;
            cache_write += ((*index != point1) as usize) & ((*index != point2) as usize) & ((*index != point3) as usize);
        }

        std::mem::swap(&mut cache, &mut cache_new);
        cache_count = if cache_write > VERTEX_CACHE_SIZE { VERTEX_CACHE_SIZE } else { cache_write };

        for index_index in 0..3 {
            let vertex_index = indices[current_triangle * 3 + index_index];

            let neighbors = &mut adjacency.data[adjacency.offsets[vertex_index]..];
            let neighbors_size = adjacency.counts[vertex_index];

            for neighbor_index in 0..neighbors_size {
                let triangle = neighbors[neighbor_index];

                if triangle == current_triangle {
                    neighbors[neighbor_index] = neighbors[neighbors_size - 1];
                    adjacency.counts[vertex_index] -= 1;
                    break;
                }
            }
        }

        let mut best_triangle = 0;
        let mut best_score = 0.0;

        for (index, cached_index) in cache.iter().enumerate().take(cache_write) {
            if adjacency.counts[*cached_index] == 0 {
                continue;
            }

            let cache_position = if index >= VERTEX_CACHE_SIZE { None } else { Some(index) };
            let score = calculate_vertex_scores(cache_position, adjacency.counts[*cached_index]);
            let score_difference = score - vertex_scores[*cached_index];

            vertex_scores[*cached_index] = score;

            for triangle_index in &adjacency.data[adjacency.offsets[*cached_index]..adjacency.offsets[*cached_index] + adjacency.counts[*cached_index]] {
                let triangle_score = triangle_scores[*triangle_index] + score_difference;

                best_triangle = if best_score < triangle_score { *triangle_index } else { best_triangle };
                best_score = if best_score < triangle_score { triangle_score } else { best_score };

                triangle_scores[*triangle_index] = triangle_score;
            }
        }

        current_triangle = best_triangle;
        if current_triangle == 0 {
            current_triangle = get_next_triangle_dead_end(&mut input_cursor, &emitted_flags, list.triangles.len());
        }
    }
}

#[derive(Debug, Default)]
struct TriangleAdjacency {
    counts: Vec<usize>,
    offsets: Vec<usize>,
    data: Vec<usize>,
}

fn build_triangle_adjacency(vertex_count: usize, indices: &[usize]) -> TriangleAdjacency {
    let mut adjacency = TriangleAdjacency {
        counts: vec![0; vertex_count],
        offsets: vec![0; vertex_count],
        data: vec![0; indices.len()],
    };

    for index in indices {
        adjacency.counts[*index] += 1;
    }

    let mut offset = 0;
    for vertex_index in 0..vertex_count {
        adjacency.offsets[vertex_index] = offset;
        offset += adjacency.counts[vertex_index];
    }

    let triangle_count = indices.len() / 3;
    for triangle_index in 0..triangle_count {
        let point1 = indices[triangle_index * 3];
        adjacency.data[adjacency.offsets[point1]] = triangle_index;
        adjacency.offsets[point1] += 1;

        let point2 = indices[triangle_index * 3 + 1];
        adjacency.data[adjacency.offsets[point2]] = triangle_index;
        adjacency.offsets[point2] += 1;

        let point3 = indices[triangle_index * 3 + 2];
        adjacency.data[adjacency.offsets[point3]] = triangle_index;
        adjacency.offsets[point3] += 1;
    }

    for vertex_index in 0..vertex_count {
        adjacency.offsets[vertex_index] -= adjacency.counts[vertex_index];
    }

    adjacency
}

const VALENCE_MAX: usize = 8;

#[derive(Debug, Default)]
struct VertexScoreTable {
    cache: [f64; 1 + VERTEX_CACHE_SIZE],
    live: [f64; 1 + VALENCE_MAX],
}

const VERTEX_SCORE_TABLE: VertexScoreTable = VertexScoreTable {
    cache: [
        0.0, 0.779, 0.791, 0.789, 0.981, 0.843, 0.726, 0.847, 0.882, 0.867, 0.799, 0.642, 0.613, 0.6, 0.568, 0.372, 0.234,
    ],
    live: [0.0, 0.995, 0.713, 0.450, 0.404, 0.059, 0.005, 0.147, 0.006],
};

fn calculate_vertex_scores(cache_position: Option<usize>, live_triangle: usize) -> f64 {
    let live_triangle_clamped = if live_triangle < VALENCE_MAX { live_triangle } else { VALENCE_MAX };

    VERTEX_SCORE_TABLE.cache[1usize.wrapping_add(cache_position.unwrap_or(usize::MAX))] + VERTEX_SCORE_TABLE.live[live_triangle_clamped]
}

fn get_next_triangle_dead_end(input_cursor: &mut usize, emitted_flags: &[bool], face_count: usize) -> usize {
    while *input_cursor < face_count {
        if !emitted_flags[*input_cursor] {
            return *input_cursor;
        }

        *input_cursor += 1;
    }

    usize::MAX
}

// Took from https://github.com/zeux/meshoptimizer/blob/master/src/overdrawoptimizer.cpp
// fn optimize_triangle_order() {}
