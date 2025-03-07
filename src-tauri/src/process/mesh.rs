use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};
use kdtree::{distance::squared_euclidean, KdTree};
use tauri::State;
use thiserror::Error as ThisError;

use crate::{
    import::{FileManager, ImportFileData, ImportVertex},
    input::ImputedCompilationData,
    process::{
        ProcessedHardwareBone, ProcessedMeshVertex, ProcessedStrip, ProcessedStripGroup, ProcessedVertex, MAX_HARDWARE_BONES_PER_STRIP, VERTEX_CACHE_SIZE,
    },
    utilities::{
        logging::{log, LogLevel},
        mathematics::{BoundingBox, Vector2, Vector3, Vector4},
    },
};

use super::{ProcessedBodyPart, ProcessedBoneData, ProcessedMesh, ProcessedModel, ProcessedModelData, FLOAT_TOLERANCE};

#[derive(Debug, ThisError)]
pub enum ProcessingMeshError {
    #[error("Model File Source Not Loaded")]
    FileSourceNotLoaded,
    #[error("Part Not Found: {0}")]
    PartNotFound(String),
    #[error("Face Was Incomplete")]
    IncompleteFace,
    #[error("Model Has Too Many Materials")]
    TooManyMaterials,
    #[error("Model Has Too Many Body Parts")]
    TooManyBodyParts,
}

#[derive(Debug, Default)]
struct WeightLink {
    bone: u8,
    weight: f64,
}

#[derive(Debug, Default)]
struct TriangleVertex {
    position: Vector3,
    normal: Vector3,
    texture_coordinate: Vector2,
    links: Vec<WeightLink>,
}

#[derive(Debug)]
struct TriangleList {
    vertices: Vec<TriangleVertex>,
    vertex_tree: KdTree<f64, usize, [f64; 3]>,
    tangents: Vec<Vector4>,
    triangles: Vec<[usize; 3]>,
}

impl Default for TriangleList {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            vertex_tree: KdTree::new(3),
            tangents: Vec::new(),
            triangles: Vec::new(),
        }
    }
}

pub fn process_meshes(
    input: &ImputedCompilationData,
    import: &State<FileManager>,
    processed_bone_data: &ProcessedBoneData,
) -> Result<ProcessedModelData, ProcessingMeshError> {
    let mut processed_model_data = ProcessedModelData::default();

    let mut bounding_box = BoundingBox::default();
    for (imputed_body_part_name, imputed_body_part) in &input.body_parts {
        let mut processed_body_part = ProcessedBodyPart {
            name: imputed_body_part_name.clone(),
            ..Default::default()
        };

        for (imputed_model_name, imputed_model) in &imputed_body_part.models {
            if imputed_model.is_blank {
                processed_body_part.models.push(ProcessedModel::default());
                continue;
            }

            let mut processed_model = ProcessedModel {
                name: imputed_model_name.clone(),
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

            let triangle_lists = create_triangle_lists(
                &imputed_model.part_names,
                imported_file,
                &mut processed_model_data.materials,
                processed_bone_data,
            )?;

            if triangle_lists.is_empty() {
                log("Model Had No Parts! Defaulting To Blank!", LogLevel::Warn);
                processed_body_part.models.push(ProcessedModel::default());
                continue;
            }

            let mut bad_vertex_count = 0;
            let mut culled_vertex_count = 0;
            let mut face_count = 0;
            let mut vertex_count = 0;
            for (material_index, mut triangle_list) in triangle_lists {
                reorder_triangle_vertex_order(&mut triangle_list);
                sort_vertices_by_hardware_bones(&mut triangle_list);
                optimize_vertex_cache(&mut triangle_list);
                // optimize_overdraw(&mut triangle_list); // FIXME: This is broken!
                bad_vertex_count += calculate_vertex_tangents(&mut triangle_list);
                culled_vertex_count += cull_weight_links(&mut triangle_list);
                let meshes = convert_to_meshes(material_index, triangle_list, &mut bounding_box);
                face_count += meshes.1;
                vertex_count += meshes.2;
                processed_model.meshes.extend(meshes.0);
            }

            if bad_vertex_count > 0 {
                log(format!("{} Had {} Bad Vertices!", imputed_model_name, bad_vertex_count), LogLevel::Warn);
            }

            if culled_vertex_count > 0 {
                log(
                    format!("{} Had {} Weight Culled Vertices!", imputed_model_name, culled_vertex_count),
                    LogLevel::Warn,
                );
            }

            log(
                format!(
                    "{} has {} faces, {} vertices and {} indices",
                    imputed_model_name,
                    face_count,
                    vertex_count,
                    face_count * 3
                ),
                LogLevel::Verbose,
            );

            processed_body_part.models.push(processed_model);
        }

        processed_model_data.body_parts.push(processed_body_part);
    }

    if processed_model_data.body_parts.len() > i32::MAX as usize {
        return Err(ProcessingMeshError::TooManyBodyParts);
    }

    if processed_model_data.materials.len() > (i16::MAX as usize + 1) {
        return Err(ProcessingMeshError::TooManyMaterials);
    }

    // TODO: Check if bounding box is too large

    processed_model_data.bounding_box = bounding_box; // TODO: Overwrite this with input bounding box.

    Ok(processed_model_data)
}

/// Combines parts into triangle lists for each material.
fn create_triangle_lists(
    part_names: &[String],
    imported_file: Arc<ImportFileData>,
    material_table: &mut IndexSet<String>,
    processed_bone_data: &ProcessedBoneData,
) -> Result<IndexMap<usize, TriangleList>, ProcessingMeshError> {
    let mut triangle_lists: IndexMap<usize, TriangleList> = IndexMap::new();

    for imputed_part_name in part_names {
        let import_part = match imported_file.parts.get(imputed_part_name) {
            Some(part) => part,
            None => return Err(ProcessingMeshError::PartNotFound(imputed_part_name.clone())),
        };

        for (material, faces) in &import_part.polygons {
            let material_index = material_table.insert_full(material.clone()).0;

            let triangle_list = triangle_lists.entry(material_index).or_default();

            for face in faces {
                if face.len() < 3 {
                    return Err(ProcessingMeshError::IncompleteFace);
                }

                let triangulated_face = triangulate_face(face, &import_part.vertices);

                for mut triangle in triangulated_face {
                    for vertex_index in &mut triangle {
                        let import_vertex = &import_part.vertices[*vertex_index];

                        let mut mapped_links = Vec::with_capacity(import_vertex.links.len());

                        for (link, weight) in &import_vertex.links {
                            let (import_bone_name, _) = imported_file.skeleton.get_index(*link).unwrap();

                            let mapped_index = match processed_bone_data.processed_bones.get_full(import_bone_name) {
                                Some((index, _, _)) => index,
                                None => todo!("Find the collapsed bone index!"),
                            };

                            mapped_links.push(WeightLink {
                                bone: mapped_index.try_into().unwrap(),
                                weight: *weight,
                            });
                        }

                        let triangle_vertex = TriangleVertex {
                            position: import_vertex.position,
                            normal: import_vertex.normal.normalize(),
                            texture_coordinate: import_vertex.texture_coordinate,
                            links: mapped_links,
                        };

                        let neighbors = triangle_list
                            .vertex_tree
                            .within(&triangle_vertex.position.as_slice(), FLOAT_TOLERANCE, &squared_euclidean)
                            .unwrap();

                        if let Some(&(_, index)) = neighbors.iter().find(|(_, &i)| vertex_equals(&triangle_vertex, &triangle_list.vertices[i])) {
                            *vertex_index = *index;
                            continue;
                        }

                        triangle_list
                            .vertex_tree
                            .add(triangle_vertex.position.as_slice(), triangle_list.vertices.len())
                            .unwrap();

                        *vertex_index = triangle_list.vertices.len();
                        triangle_list.vertices.push(triangle_vertex);
                    }

                    triangle_list.triangles.push(triangle);
                }
            }
        }
    }

    Ok(triangle_lists)
}

/// Triangulates a face into a triangles.
fn triangulate_face(face: &[usize], vertices: &[ImportVertex]) -> Vec<[usize; 3]> {
    if face.len() == 3 {
        return vec![[face[0], face[1], face[2]]];
    }

    if face.len() == 4 {
        return vec![[face[0], face[1], face[2]], [face[2], face[3], face[0]]];
    }

    // TODO: Implement a better triangulation algorithm.

    let mut triangles = Vec::new();

    let index_count = face.len();
    let mut minimum_distance = f64::MAX;
    let mut minimum_index = 0;

    let loop_count = match index_count {
        count if count > 4 => count,
        4 => 2,
        _ => 0,
    };

    for loop_index in 0..loop_count {
        let mut distance = 0.0;

        let center = vertices[face[loop_index]].position;
        for distance_loop_index in 2..index_count - 1 {
            let edge_index = (loop_index + distance_loop_index) % index_count;
            let edge = vertices[face[edge_index]].position;
            distance += (edge - center).magnitude();
        }

        if distance < minimum_distance {
            minimum_index = loop_index;
            minimum_distance = distance;
        }
    }

    for triangle_build_index in 1..index_count - 1 {
        triangles.push([
            face[minimum_index],
            face[(minimum_index + triangle_build_index) % index_count],
            face[(minimum_index + triangle_build_index + 1) % index_count],
        ]);
    }

    triangles
}

/// Compares two triangle vertices for equality.
fn vertex_equals(from: &TriangleVertex, to: &TriangleVertex) -> bool {
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

    if from
        .links
        .iter()
        .zip(to.links.iter())
        .any(|(from_link, to_link)| from_link.bone != to_link.bone || from_link.weight != to_link.weight)
    {
        return false;
    }

    true
}

/// Reorders the triangle vertex order to be clockwise.
fn reorder_triangle_vertex_order(triangle_list: &mut TriangleList) {
    // TODO: Actually implement this function if a file format has a clockwise format.
    for triangle in &mut triangle_list.triangles {
        triangle.reverse();
    }
}

/// Sorts the vertices to decrease the amount of strips.
fn sort_vertices_by_hardware_bones(_triangle_list: &mut TriangleList) {
    // TODO: Implement this function.
}

/// Sorts the indices to decrease the amount of cache misses.
/// Implementation of https://github.com/zeux/meshoptimizer/blob/master/src/vcacheoptimizer.cpp
fn optimize_vertex_cache(triangle_list: &mut TriangleList) {
    const VERTEX_VALENCE_SIZE: usize = 8;

    struct VertexScoreTable {
        cache: [f64; 1 + VERTEX_CACHE_SIZE],
        live: [f64; 1 + VERTEX_VALENCE_SIZE],
    }

    const VERTEX_SCORE_TABLE: VertexScoreTable = VertexScoreTable {
        cache: [
            0.0, 0.779, 0.791, 0.789, 0.981, 0.843, 0.726, 0.847, 0.882, 0.867, 0.799, 0.642, 0.613, 0.600, 0.568, 0.372, 0.234,
        ],
        live: [0.0, 0.995, 0.713, 0.450, 0.404, 0.059, 0.005, 0.147, 0.006],
    };

    struct TriangleAdjacency {
        counts: Vec<usize>,
        offsets: Vec<usize>,
        data: Vec<usize>,
    }

    let triangle_count = triangle_list.triangles.len();
    let indices = triangle_list.triangles.drain(..).flatten().collect::<Vec<_>>();

    let mut adjacency = TriangleAdjacency {
        counts: vec![0; triangle_list.vertices.len()],
        offsets: vec![0; triangle_list.vertices.len()],
        data: vec![0; indices.len()],
    };

    for &vertex_index in &indices {
        adjacency.counts[vertex_index] += 1;
    }

    let mut offset = 0;
    for offset_index in 0..adjacency.offsets.len() {
        adjacency.offsets[offset_index] = offset;
        offset += adjacency.counts[offset_index];
    }

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

    for offset_fix_index in 0..adjacency.offsets.len() {
        adjacency.offsets[offset_fix_index] -= adjacency.counts[offset_fix_index];
    }

    fn calculate_vertex_score(cache_position: Option<usize>, live_triangle: usize) -> f64 {
        let live_triangles_clamped = if live_triangle < VERTEX_VALENCE_SIZE {
            live_triangle
        } else {
            VERTEX_VALENCE_SIZE
        };

        VERTEX_SCORE_TABLE.cache[cache_position.unwrap_or_default()] + VERTEX_SCORE_TABLE.live[live_triangles_clamped]
    }
    let mut vertex_scores = vec![0.0; triangle_list.vertices.len()];
    for (vertex_index, vertex_score) in vertex_scores.iter_mut().enumerate() {
        *vertex_score = calculate_vertex_score(None, adjacency.counts[vertex_index]);
    }

    let mut triangle_scores = vec![0.0; triangle_count];
    for (triangle_index, triangle_score) in triangle_scores.iter_mut().enumerate() {
        let point1 = indices[triangle_index * 3];
        let point2 = indices[triangle_index * 3 + 1];
        let point3 = indices[triangle_index * 3 + 2];
        *triangle_score = vertex_scores[point1] + vertex_scores[point2] + vertex_scores[point3];
    }

    let mut cache = [0; VERTEX_CACHE_SIZE + 4];
    let mut cache_new = [0; VERTEX_CACHE_SIZE + 4];
    let mut cache_count = 0;

    let mut current_triangle = Some(0);
    let mut input_cursor = 1;

    let mut emitted_flags = vec![false; triangle_count];

    while current_triangle.is_some() {
        let a = indices[current_triangle.unwrap() * 3];
        let b = indices[current_triangle.unwrap() * 3 + 1];
        let c = indices[current_triangle.unwrap() * 3 + 2];

        triangle_list.triangles.push([a, b, c]);

        emitted_flags[current_triangle.unwrap()] = true;
        triangle_scores[current_triangle.unwrap()] = 0.0;

        let mut cache_write = 0;
        cache_new[cache_write] = a;
        cache_write += 1;
        cache_new[cache_write] = b;
        cache_write += 1;
        cache_new[cache_write] = c;
        cache_write += 1;

        for &cache_value in cache.iter().take(cache_count) {
            cache_new[cache_write] = cache_value;
            cache_write += ((cache_value != a) as usize) & ((cache_value != b) as usize) & ((cache_value != c) as usize);
        }

        std::mem::swap(&mut cache, &mut cache_new);
        cache_count = if cache_write > VERTEX_CACHE_SIZE { VERTEX_CACHE_SIZE } else { cache_write };

        for vertex_index in 0..3 {
            let neighbors = &mut adjacency.data[adjacency.offsets[vertex_index]..];
            let neighbors_size = adjacency.counts[vertex_index];

            for neighbor_index in 0..neighbors_size {
                let neighbor_triangle = neighbors[neighbor_index];

                if neighbor_triangle == current_triangle.unwrap() {
                    neighbors[neighbor_index] = neighbors[neighbors_size - 1];
                    adjacency.counts[vertex_index] -= 1;
                    break;
                }
            }
        }

        let mut best_triangle = None;
        let mut best_score = 0.0;

        for &cache_value in cache.iter().take(cache_write) {
            if adjacency.counts[cache_value] == 0 {
                continue;
            }

            let cache_position = if cache_value >= VERTEX_CACHE_SIZE { None } else { Some(cache_value) };
            let score = calculate_vertex_score(cache_position, adjacency.counts[cache_value]);
            let score_difference = score - vertex_scores[cache_value];

            vertex_scores[cache_value] = score;

            for &triangle_index in &adjacency.data[adjacency.offsets[cache_value]..adjacency.offsets[cache_value] + adjacency.counts[cache_value]] {
                let triangle_score = triangle_scores[triangle_index] + score_difference;

                best_triangle = if best_score < triangle_score { Some(triangle_index) } else { None };
                best_score = if best_score < triangle_score { triangle_score } else { best_score };

                triangle_scores[triangle_index] = triangle_score;
            }
        }

        current_triangle = best_triangle;
        if current_triangle.is_none() {
            while input_cursor < triangle_count {
                if !emitted_flags[input_cursor] {
                    current_triangle = Some(input_cursor);
                    break;
                }

                input_cursor += 1;
            }

            if input_cursor == triangle_count {
                current_triangle = None;
            }
        }
    }
}

/// Sorts the indices to decrease the amount of overdraw.
/// Implementation of https://github.com/zeux/meshoptimizer/blob/master/src/overdrawoptimizer.cpp
fn _optimize_overdraw(triangle_list: &mut TriangleList) {
    // TODO: Configure threshold to work well with source or make it a parameter.
    let threshold = 1.05;

    let triangle_count = triangle_list.triangles.len();
    let indices = triangle_list.triangles.drain(..).flatten().collect::<Vec<_>>();

    fn update_cache(a: usize, b: usize, c: usize, cache_timestamps: &mut [usize], timestamp: &mut usize) -> usize {
        let mut cache_misses = 0;

        if *timestamp - cache_timestamps[a] > VERTEX_CACHE_SIZE {
            cache_timestamps[a] = *timestamp;
            *timestamp += 1;
            cache_misses += 1;
        }

        if *timestamp - cache_timestamps[b] > VERTEX_CACHE_SIZE {
            cache_timestamps[b] = *timestamp;
            *timestamp += 1;
            cache_misses += 1;
        }

        if *timestamp - cache_timestamps[c] > VERTEX_CACHE_SIZE {
            cache_timestamps[c] = *timestamp;
            *timestamp += 1;
            cache_misses += 1;
        }

        cache_misses
    }

    let mut cache_timestamps = vec![0; triangle_list.vertices.len()];
    let mut timestamp = VERTEX_CACHE_SIZE + 1;
    let mut hard_clusters = vec![0; indices.len() / 3];
    let mut hard_cluster_count = 0;
    for triangle_index in 0..triangle_count {
        let a = indices[triangle_index * 3];
        let b = indices[triangle_index * 3 + 1];
        let c = indices[triangle_index * 3 + 2];

        let cache_misses = update_cache(a, b, c, &mut cache_timestamps, &mut timestamp);

        if triangle_index == 0 || cache_misses == 3 {
            hard_clusters[hard_cluster_count] = triangle_index;
            hard_cluster_count += 1;
        }
    }

    let mut cache_timestamps = vec![0; triangle_list.vertices.len()];
    let mut timestamp = 0;
    let mut soft_clusters = vec![0; indices.len() / 3 + 1];
    let mut soft_cluster_count = 0;
    for it in 0..hard_cluster_count {
        let start = hard_clusters[it];
        let end = if it + 1 < hard_cluster_count {
            hard_clusters[it + 1]
        } else {
            indices.len() / 3
        };

        debug_assert!(start < end);

        timestamp += VERTEX_CACHE_SIZE + 1;

        let mut cluster_misses = 0;
        for cache_vertex in start..end {
            let a = indices[cache_vertex * 3];
            let b = indices[cache_vertex * 3 + 1];
            let c = indices[cache_vertex * 3 + 2];

            cluster_misses += update_cache(a, b, c, &mut cache_timestamps, &mut timestamp);
        }

        let cluster_threshold = threshold * (cluster_misses as f64 / (end - start) as f64);

        soft_clusters[soft_cluster_count] = start;
        soft_cluster_count += 1;

        timestamp += VERTEX_CACHE_SIZE + 1;

        let mut running_misses = 0;
        let mut running_faces = 0;

        for cache_vertex in start..end {
            let a = indices[cache_vertex * 3];
            let b = indices[cache_vertex * 3 + 1];
            let c = indices[cache_vertex * 3 + 2];

            running_misses += update_cache(a, b, c, &mut cache_timestamps, &mut timestamp);
            running_faces += 1;

            if running_misses as f64 / running_faces as f64 <= cluster_threshold {
                soft_clusters[soft_cluster_count] = cache_vertex + 1;
                soft_cluster_count += 1;

                timestamp += VERTEX_CACHE_SIZE + 1;

                running_misses = 0;
                running_faces = 0;
            }
        }

        if soft_clusters[soft_cluster_count - 1] != start {
            soft_cluster_count -= 1;
        }
    }

    debug_assert!(soft_cluster_count >= hard_cluster_count);
    debug_assert!(soft_cluster_count <= indices.len() / 3);

    let clusters = &soft_clusters;
    let cluster_count = soft_cluster_count;

    let mut sort_data = vec![0.0; cluster_count];

    let mut mesh_centroid = Vector3::default();

    for vertex_index in &indices {
        let vertex = &triangle_list.vertices[*vertex_index];
        mesh_centroid = mesh_centroid + vertex.position;
    }

    mesh_centroid = mesh_centroid / indices.len() as f64;

    for cluster in 0..cluster_count {
        let cluster_begin = clusters[cluster] * 3;
        let cluster_end = if cluster + 1 < cluster_count {
            clusters[cluster + 1] * 3
        } else {
            indices.len()
        };
        debug_assert!(cluster_begin < cluster_end, "{} < {}", cluster_begin, cluster_end);

        let mut cluster_area = 0.0;
        let mut cluster_centroid = Vector3::default();
        let mut cluster_normal = Vector3::default();

        for vertex_index in (cluster_begin..cluster_end).step_by(3) {
            let position0 = triangle_list.vertices[indices[vertex_index]].position;
            let position1 = triangle_list.vertices[indices[vertex_index + 1]].position;
            let position2 = triangle_list.vertices[indices[vertex_index + 2]].position;

            let position10 = position1 - position0;
            let position20 = position2 - position0;

            let normal = position10.cross(position20);

            let area = normal.magnitude();

            cluster_centroid = cluster_centroid + ((position0 + position1 + position2) * (area / 3.0));
            cluster_normal = cluster_normal + normal;
            cluster_area += area;
        }

        let inverse_cluster_area = if cluster_area == 0.0 { 0.0 } else { 1.0 / cluster_area };

        cluster_centroid = cluster_centroid * inverse_cluster_area;

        let cluster_normal_length = cluster_normal.magnitude();
        let inverse_cluster_normal_length = if cluster_normal_length == 0.0 { 0.0 } else { 1.0 / cluster_normal_length };

        cluster_normal = cluster_normal * inverse_cluster_normal_length;

        let centroid_vector = cluster_centroid - mesh_centroid;
        sort_data[cluster] = centroid_vector.dot(cluster_normal);
    }

    let mut sort_keys = vec![0; cluster_count];
    let mut sort_order = vec![0; cluster_count];

    let mut sort_data_max = f64::MIN;

    for sort_data_value in &sort_data {
        sort_data_max = sort_data_max.max(sort_data_value.abs());
    }

    const SORT_BITS: usize = 11;

    for cluster in 0..cluster_count {
        let mut sort_key = 0.5 - 0.5 * (sort_data[cluster] / sort_data_max);

        let scale = ((1 << SORT_BITS) - 1) as f64;

        sort_key = if sort_key >= 0.0 { sort_key } else { 0.0 };
        sort_key = if sort_key <= 1.0 { sort_key } else { 1.0 };

        let quantize = (sort_key * scale + 0.5) as usize;

        sort_keys[cluster] = quantize & ((1 << SORT_BITS) - 1);
    }

    let mut histogram = [0; 1 << SORT_BITS];

    for &sort_key in &sort_keys {
        histogram[sort_key] += 1;
    }

    let mut histogram_sum = 0;

    for histogram_value in histogram.iter_mut() {
        let count = *histogram_value;
        *histogram_value = histogram_sum;
        histogram_sum += count;
    }

    debug_assert!(histogram_sum == cluster_count);

    for (cluster, &sort_key) in sort_keys.iter().enumerate() {
        sort_order[histogram[sort_key]] = cluster;
        histogram[sort_key] += 1;
    }

    let mut offset = 0;
    for cluster in sort_order {
        debug_assert!(cluster < cluster_count);

        let cluster_begin = clusters[cluster] * 3;
        let cluster_end = if cluster + 1 < cluster_count {
            clusters[cluster + 1] * 3
        } else {
            indices.len()
        };
        debug_assert!(cluster_begin < cluster_end);

        for vertex_index in (cluster_begin..cluster_end).step_by(3) {
            triangle_list
                .triangles
                .push([indices[vertex_index], indices[vertex_index + 1], indices[vertex_index + 2]]);
        }

        offset += cluster_end - cluster_begin;
    }
    debug_assert!(offset == indices.len());
}

/// Calculates the tangents for each vertex.
fn calculate_vertex_tangents(triangle_list: &mut TriangleList) -> usize {
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
            bi_tangents[face[vertex_index]] = bi_tangents[face[vertex_index]] + bi_tangent;
        }
    }

    triangle_list.tangents.reserve(triangle_list.vertices.len());
    let mut bad_vertex_count = 0;
    for index in 0..triangle_list.vertices.len() {
        let normalized_tangent = tangents[index].normalize();
        let normalized_bi_tangent = bi_tangents[index].normalize();

        let cross_product = triangle_list.vertices[index].normal.cross(normalized_tangent);
        let sign = if cross_product.dot(normalized_bi_tangent) < 0.0 { -1.0 } else { 1.0 };

        let vertex_tangent = Vector4::new(normalized_tangent.x, normalized_tangent.y, normalized_tangent.z, sign);

        triangle_list.tangents.push(vertex_tangent);

        // This is what source considers a bad vertex.
        // TODO: Find a better way to calculate vertex tangents to not have bad vertices.
        let tangent_dot = normalized_tangent.dot(triangle_list.vertices[index].normal);
        if !(-0.95..=0.95).contains(&tangent_dot) {
            bad_vertex_count += 1;
        }
    }
    bad_vertex_count
}

/// Culls the weight links to a maximum of 3.
fn cull_weight_links(triangle_list: &mut TriangleList) -> usize {
    let mut culled_vertex_count = 0;

    for vertex in &mut triangle_list.vertices {
        if vertex.links.len() <= 3 {
            continue;
        }

        vertex.links.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        vertex.links.truncate(3);
        culled_vertex_count += 1;
    }

    culled_vertex_count
}

/// Converts a triangle list into a list of processed meshes.
fn convert_to_meshes(material_index: usize, triangle_list: TriangleList, bounding_box: &mut BoundingBox) -> (Vec<ProcessedMesh>, usize, usize) {
    let mut processed_meshes = Vec::new();

    let mut processed_mesh = ProcessedMesh {
        material: material_index.try_into().unwrap(),
        ..Default::default()
    };

    let mut triangle_count = 0;
    let mut vertex_count = 0;

    let mut processed_strip_group = ProcessedStripGroup::default();
    let mut processed_strip = ProcessedStrip::default();

    let mut mapped_indices: IndexMap<usize, usize> = IndexMap::new();
    let mut hardware_bones: IndexSet<u8> = IndexSet::new();
    for triangle in triangle_list.triangles {
        let new_vertex_count = triangle
            .iter()
            .filter_map(|&value| if !mapped_indices.contains_key(&value) { Some(value) } else { None })
            .collect::<Vec<usize>>();

        let unique_new_vertices = new_vertex_count.iter().collect::<IndexSet<_>>();

        let new_hardware_bone_count = triangle
            .iter()
            .map(|index| &triangle_list.vertices[*index])
            .flat_map(|vertex| vertex.links.iter().filter(|link| !hardware_bones.contains(&link.bone)))
            .map(|link| link.bone)
            .collect::<Vec<u8>>();

        let unique_new_hardware_bones = new_hardware_bone_count.iter().collect::<IndexSet<_>>();

        if processed_strip_group.vertices.len() + unique_new_vertices.len() > (u16::MAX as usize + 1) {
            processed_strip_group.strips.push(processed_strip);
            processed_mesh.strip_groups.push(processed_strip_group);
            processed_meshes.push(processed_mesh);

            mapped_indices.clear();
            hardware_bones.clear();

            processed_mesh = ProcessedMesh {
                material: material_index.try_into().unwrap(),
                ..Default::default()
            };
            processed_strip_group = ProcessedStripGroup::default();
            processed_strip = ProcessedStrip::default();
        }

        if hardware_bones.len() + unique_new_hardware_bones.len() > MAX_HARDWARE_BONES_PER_STRIP {
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
                processed_strip_group.indices.push((*mapped_indices.get(&index).unwrap()).try_into().unwrap());
                processed_strip.indices_count += 1;
                continue;
            }

            let vertex_data = &triangle_list.vertices[index];

            let mut vertex_weights = [0.0; 3];
            let mut weight_bones = [0; 3];
            let mut weight_count = 0;

            for link in &vertex_data.links {
                debug_assert!(weight_count < 3, "Vertex has more than 3 weights!");

                // Merge links with the same bone
                if let Some(existing_link) = weight_bones.iter().take(weight_count).position(|&bone| bone == link.bone) {
                    vertex_weights[existing_link] += link.weight as f32;
                    continue;
                }

                vertex_weights[weight_count] = link.weight as f32;
                weight_bones[weight_count] = link.bone;

                weight_count += 1;
            }

            debug_assert!(weight_count > 0, "Vertex has no weights!");

            let sum = vertex_weights.iter().take(weight_count).sum::<f32>();

            for weight in vertex_weights.iter_mut().take(weight_count) {
                *weight /= sum;
            }

            let processed_vertex = ProcessedVertex {
                weights: vertex_weights,
                bones: weight_bones,
                bone_count: weight_count as u8,
                position: vertex_data.position,
                normal: vertex_data.normal,
                texture_coordinate: vertex_data.texture_coordinate,
                tangent: triangle_list.tangents[index],
            };

            bounding_box.add_point(processed_vertex.position);

            let mut processed_mesh_vertex = ProcessedMeshVertex {
                vertex_index: processed_strip_group.vertices.len().try_into().unwrap(),
                bone_count: weight_count as u8,
                ..Default::default()
            };

            processed_strip.bone_count = if weight_count as i16 > processed_strip.bone_count {
                weight_count as i16
            } else {
                processed_strip.bone_count
            };

            for (bone_index, bone) in weight_bones.iter().enumerate().take(weight_count) {
                let (hardware_bone_index, new_hardware_bone) = hardware_bones.insert_full(*bone);

                processed_mesh_vertex.bones[bone_index] = hardware_bone_index.try_into().unwrap();
                if new_hardware_bone {
                    let processed_hardware_bone = ProcessedHardwareBone {
                        hardware_bone: hardware_bone_index.try_into().unwrap(),
                        bone_table_bone: *bone as i32,
                    };
                    processed_strip.hardware_bones.push(processed_hardware_bone);
                }
            }

            processed_strip_group.indices.push(processed_strip_group.vertices.len().try_into().unwrap());
            mapped_indices.insert(index, processed_strip_group.vertices.len());
            processed_strip.indices_count += 1;

            processed_strip_group.vertices.push(processed_mesh_vertex);
            processed_mesh.vertex_data.push(processed_vertex);
            processed_strip.vertex_count += 1;

            vertex_count += 1;
        }

        triangle_count += 1;
    }

    processed_strip_group.strips.push(processed_strip);
    processed_mesh.strip_groups.push(processed_strip_group);
    processed_meshes.push(processed_mesh);

    (processed_meshes, triangle_count, vertex_count)
}
