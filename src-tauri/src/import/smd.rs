use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::utilities::{
    logging::{log, LogLevel},
    mathematics::{Angles, Vector2, Vector3},
};

use super::{ImportedAnimationFrame, ImportedBone, ImportedBoneAnimation, ImportedFace, ImportedFileData, ImportedVertex};

#[derive(Debug)]
pub enum SMDParseError {
    NotValidUTF8,
    InvalidVersion,
    InvalidNumber,
}

impl Display for SMDParseError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let error_message: &str = match self {
            SMDParseError::NotValidUTF8 => "Line Was Not Valid UTF8!",
            SMDParseError::InvalidVersion => "SMD Version Not Vaild!",
            SMDParseError::InvalidNumber => "Number Was Not Valid!",
        };

        fmt.write_str(error_message)
    }
}

impl Error for SMDParseError {}

pub fn load_smd(file_path: &Path, vta_path: Option<&Path>) -> Result<ImportedFileData, SMDParseError> {
    log(
        format!("Loading SMD File: {}", file_path.file_name().unwrap().to_str().unwrap()), // UNWRAP: Path should be valid from caller.
        LogLevel::Verbose,
    );

    let file = File::open(file_path).unwrap(); // UNWRAP: The file should be accessible from caller.
    let file_buffer = BufReader::new(file);
    let mut file_lines = file_buffer.lines();
    let mut line_count: usize = 0;

    let mut file_data = SMDFileData::new();

    loop {
        line_count += 1;
        let next_line = match file_lines.next() {
            Some(line) => match line {
                Ok(line) => line,
                Err(error) => {
                    log(format!("Failed On Line: {}", line_count), LogLevel::Verbose);
                    log(format!("Fail Reason: {}", error), LogLevel::Debug);
                    return Err(SMDParseError::NotValidUTF8);
                }
            },
            None => break,
        };

        let mut line_arguments = next_line.split_whitespace();

        match line_arguments.next() {
            Some("version") => {
                let version = match line_arguments.next() {
                    Some(argument) => match argument.parse::<isize>() {
                        Ok(version) => version,
                        Err(_) => todo!("Log and return with parse error!"),
                    },
                    None => todo!("Log and return with missing version argument!"),
                };

                if version < 1 || version > 3 {
                    log(format!("Failed On Line: {}", line_count), LogLevel::Verbose);
                    return Err(SMDParseError::InvalidVersion);
                }

                log(format!("SMD Version: {}", version), LogLevel::Debug);

                file_data.version = version
            }
            Some("nodes") => loop {
                line_count += 1;
                let next_line = match file_lines.next() {
                    Some(line) => match line {
                        Ok(line) => line,
                        Err(error) => {
                            log(format!("Line failed to read: {}", error.to_string()), LogLevel::Error);
                            return Err(SMDParseError::NotValidUTF8);
                        }
                    },
                    None => todo!("Log and return with unexpected end of line!"),
                };

                let mut line_arguments = next_line.split_whitespace();

                let node_id = match line_arguments.next() {
                    Some(argument) => {
                        if argument == "end" {
                            break;
                        }

                        match argument.parse::<isize>() {
                            Ok(node_id) => node_id,
                            Err(_) => todo!("Log and return with parse error!"),
                        }
                    }
                    None => continue,
                };

                if node_id < 0 {}

                if file_data.nodes.iter().any(|node| node.id == node_id) {
                    todo!("Log and error node id already used!")
                }

                let node_name = match line_arguments.next() {
                    Some(argument) => {
                        if !argument.starts_with('"') || !argument.ends_with('"') {
                            todo!("Log and error node name not in quotes!")
                        }

                        if argument.len() == 2 {
                            todo!("Log and error node has no name!")
                        }

                        argument[1..argument.len() - 1].to_string()
                    }
                    None => todo!("Log and error missing node name!"),
                };

                let node_parent = match line_arguments.next() {
                    Some(argument) => match argument.parse::<isize>() {
                        Ok(node_parent) => node_parent,
                        Err(_) => todo!("Log and return with parse errro!"),
                    },
                    None => todo!(),
                };

                if node_parent < -1 {
                    todo!("Log and error node parent can't be negitve!")
                }

                if node_parent > -1 && !file_data.nodes.iter().any(|node| node.id == node_parent) {
                    todo!("Log and error node could not find parent!")
                }

                file_data.nodes.push(Node::new(node_id, node_name, node_parent))
            },
            Some("skeleton") => loop {
                line_count += 1;
                let next_line = match file_lines.next() {
                    Some(line) => match line {
                        Ok(line) => line,
                        Err(error) => {
                            log(format!("Line failed to read: {}", error.to_string()), LogLevel::Error);
                            return Err(SMDParseError::NotValidUTF8);
                        }
                    },
                    None => todo!("Log and return with unexpected end of line!"),
                };

                let mut line_arguments = next_line.split_whitespace();

                let node_id = match line_arguments.next() {
                    Some(argument) => {
                        if argument == "end" {
                            break;
                        }

                        if argument == "time" {
                            let time = match line_arguments.next() {
                                Some(time_argumnt) => match time_argumnt.parse::<usize>() {
                                    Ok(time) => time,
                                    Err(_) => todo!("Parse faild"),
                                },
                                None => todo!("Fame argument doesn't exist!"),
                            };

                            if time != file_data.frames.len() {
                                todo!("Frames are not sequential!")
                            }

                            file_data.frames.push(Frame::new());

                            continue;
                        }

                        match argument.parse::<isize>() {
                            Ok(node_id) => node_id,
                            Err(_) => todo!("Log and return with parse error!"),
                        }
                    }
                    None => continue,
                };

                if !file_data.nodes.iter().any(|node| node.id == node_id) {
                    todo!("Faild to find node id!")
                }

                let position_x = match line_arguments.next() {
                    Some(argument) => match argument.parse::<f64>() {
                        Ok(position) => position,
                        Err(_) => todo!(),
                    },
                    None => todo!(),
                };

                let position_y = match line_arguments.next() {
                    Some(argument) => match argument.parse::<f64>() {
                        Ok(position) => position,
                        Err(_) => todo!(),
                    },
                    None => todo!(),
                };

                let position_z = match line_arguments.next() {
                    Some(argument) => match argument.parse::<f64>() {
                        Ok(position) => position,
                        Err(_) => todo!(),
                    },
                    None => todo!(),
                };

                let rotation_x = match line_arguments.next() {
                    Some(argument) => match argument.parse::<f64>() {
                        Ok(rotation) => rotation,
                        Err(_) => todo!(),
                    },
                    None => todo!(),
                };

                let rotation_y = match line_arguments.next() {
                    Some(argument) => match argument.parse::<f64>() {
                        Ok(rotation) => rotation,
                        Err(_) => todo!(),
                    },
                    None => todo!(),
                };

                let rotation_z = match line_arguments.next() {
                    Some(argument) => match argument.parse::<f64>() {
                        Ok(rotation) => rotation,
                        Err(_) => todo!(),
                    },
                    None => todo!(),
                };

                let frame = match file_data.frames.last_mut() {
                    Some(frame) => frame,
                    None => todo!(),
                };

                frame.animated_nodes.push(AnimationNode::new(
                    node_id,
                    Vector3::new(position_x, position_y, position_z),
                    Angles::new(rotation_x, rotation_y, rotation_z),
                ))
            },
            Some("triangles") => loop {
                line_count += 1;
                let next_line = match file_lines.next() {
                    Some(line) => match line {
                        Ok(line) => line,
                        Err(error) => {
                            log(format!("Line failed to read: {}", error.to_string()), LogLevel::Error);
                            return Err(SMDParseError::NotValidUTF8);
                        }
                    },
                    None => todo!("Log and return with unexpected end of line!"),
                };

                let mut line_arguments = next_line.split_whitespace();

                let material = match line_arguments.next() {
                    Some(argument) => {
                        if argument == "end" {
                            break;
                        }

                        argument.to_string()
                    }
                    None => continue,
                };

                let material_index = match file_data.materials.iter().position(|material_name| material_name == &material) {
                    Some(index) => index,
                    None => {
                        file_data.materials.push(material);
                        file_data.materials.len() - 1
                    }
                };

                let mut triangle = Triangle::new(material_index);

                while triangle.vertices.len() < 3 {
                    line_count += 1;
                    let next_line = match file_lines.next() {
                        Some(line) => match line {
                            Ok(line) => line,
                            Err(error) => {
                                log(format!("Line failed to read: {}", error.to_string()), LogLevel::Error);
                                return Err(SMDParseError::NotValidUTF8);
                            }
                        },
                        None => todo!("Log and return with unexpected end of line!"),
                    };

                    let mut line_arguments = next_line.split_whitespace();

                    let node_id = match line_arguments.next() {
                        Some(argument) => {
                            if argument == "end" {
                                todo!("Unexpected end of triangles")
                            }

                            match argument.parse::<isize>() {
                                Ok(node_id) => node_id,
                                Err(error) => {
                                    log(format!("Failed on line {}", line_count), LogLevel::Verbose);
                                    log(format!("Fail Reason: {}", error), LogLevel::Debug);
                                    return Err(SMDParseError::InvalidNumber);
                                }
                            }
                        }
                        None => continue,
                    };

                    let position_x = match line_arguments.next() {
                        Some(argument) => match argument.parse::<f64>() {
                            Ok(position) => position,
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    };

                    let position_y = match line_arguments.next() {
                        Some(argument) => match argument.parse::<f64>() {
                            Ok(position) => position,
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    };

                    let position_z = match line_arguments.next() {
                        Some(argument) => match argument.parse::<f64>() {
                            Ok(position) => position,
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    };

                    let normal_x = match line_arguments.next() {
                        Some(argument) => match argument.parse::<f64>() {
                            Ok(normal) => normal,
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    };

                    let normal_y = match line_arguments.next() {
                        Some(argument) => match argument.parse::<f64>() {
                            Ok(normal) => normal,
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    };

                    let normal_z = match line_arguments.next() {
                        Some(argument) => match argument.parse::<f64>() {
                            Ok(normal) => normal,
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    };

                    let uv_x = match line_arguments.next() {
                        Some(argument) => match argument.parse::<f64>() {
                            Ok(uv) => uv,
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    };

                    let uv_y = match line_arguments.next() {
                        Some(argument) => match argument.parse::<f64>() {
                            Ok(uv) => uv,
                            Err(_) => todo!(),
                        },
                        None => todo!(),
                    };

                    let mut vertex = Vertex::new(
                        Vector3::new(position_x, position_y, position_z),
                        Vector3::new(normal_x, normal_y, normal_z),
                        Vector2::new(uv_x, uv_y),
                    );

                    let link_count = match line_arguments.next() {
                        Some(argument) => match argument.parse::<isize>() {
                            Ok(link_count) => link_count,
                            Err(_) => todo!(),
                        },
                        None => 0,
                    };

                    let mut weighting = 0.0;
                    for _ in 0..link_count {
                        let node_id = match line_arguments.next() {
                            Some(argument) => match argument.parse::<isize>() {
                                Ok(node_id) => node_id,
                                Err(_) => todo!(),
                            },
                            None => todo!(),
                        };

                        let weight = match line_arguments.next() {
                            Some(argument) => match argument.parse::<f64>() {
                                Ok(normal) => normal,
                                Err(_) => todo!(),
                            },
                            None => todo!(),
                        };

                        weighting += weight;
                        vertex.weights.push((node_id, weight))
                    }

                    if weighting < 1.0 {
                        vertex.weights.push((node_id, 1.0 - weighting))
                    }

                    if file_data.version < 3 {
                        triangle.vertices.push(vertex);
                        continue;
                    }

                    let uv_count = match line_arguments.next() {
                        Some(argument) => match argument.parse::<isize>() {
                            Ok(uv_count) => uv_count,
                            Err(_) => todo!(),
                        },
                        None => {
                            triangle.vertices.push(vertex);
                            continue;
                        }
                    };

                    for _ in 0..uv_count {
                        let uv_x = match line_arguments.next() {
                            Some(argument) => match argument.parse::<f64>() {
                                Ok(uv) => uv,
                                Err(_) => todo!(),
                            },
                            None => todo!(),
                        };

                        let uv_y = match line_arguments.next() {
                            Some(argument) => match argument.parse::<f64>() {
                                Ok(uv) => uv,
                                Err(_) => todo!(),
                            },
                            None => todo!(),
                        };

                        vertex.extra_uv.push(Vector2::new(uv_x, uv_y))
                    }

                    triangle.vertices.push(vertex);
                }

                file_data.triangles.push(triangle)
            },
            Some("vertexanimation") => todo!("Log and error flexes should be stored in vta file"),
            Some(_) => todo!("Log and return for unknown command!"),
            None => continue,
        }
    }

    if let Some(vta_path) = vta_path {
        todo!("Implment vta reading!")
    }

    let mut imported_file_data = ImportedFileData::new();
    let mut mapped_nodes: HashMap<isize, usize> = HashMap::new();

    let bind_pose = match file_data.frames.first() {
        Some(frame) => frame,
        None => todo!("SMd requires first frame"),
    };

    if bind_pose.animated_nodes.len() != file_data.nodes.len() {
        todo!("First frame requires all nodes")
    }

    for node in file_data.nodes {
        // UNWRAP: All check should not allow it to not exist
        let bind = bind_pose.animated_nodes.iter().find(|animation| animation.id == node.id).unwrap();

        let parent = if node.parent == -1 {
            None
        } else {
            // UNWRAP: All check should not allow it to not exist
            Some(*mapped_nodes.get(&node.parent).unwrap())
        };

        let new_bone = ImportedBone::new(node.name, bind.position, bind.rotation.to_quaternion(), parent);

        let bone_id = imported_file_data.add_bone(new_bone);

        mapped_nodes.insert(node.id, bone_id);
    }

    for frame in file_data.frames {
        let mut frame_data = ImportedAnimationFrame::new();

        for animation in frame.animated_nodes {
            // UNWRAP: All check should not allow it to not exist
            let bone_id = *mapped_nodes.get(&animation.id).unwrap();

            let animation = ImportedBoneAnimation::new(bone_id, animation.position, animation.rotation.to_quaternion());

            frame_data.add_bone(animation);
        }

        imported_file_data.add_frame(frame_data);
    }

    for material in file_data.materials {
        imported_file_data.mesh.add_material(material);
    }

    for triangle in file_data.triangles {
        let mut face = ImportedFace::new(triangle.material_index);

        for vertex in triangle.vertices {
            let mut vertice = ImportedVertex::new(vertex.position, vertex.normal, vertex.uv);

            for (bone_id, weight) in vertex.weights {
                // UNWRAP: All check should not allow it to not exist
                let bone_id = *mapped_nodes.get(&bone_id).unwrap();

                vertice.add_weight(bone_id, weight);
            }

            let vertex_index = imported_file_data.mesh.add_vertex(vertice);

            face.add_vertex_index(vertex_index);
        }

        imported_file_data.mesh.add_face(face);
    }

    Ok(imported_file_data)
}

struct SMDFileData {
    version: isize,
    nodes: Vec<Node>,
    frames: Vec<Frame>,
    materials: Vec<String>,
    triangles: Vec<Triangle>,
}

impl SMDFileData {
    fn new() -> Self {
        SMDFileData {
            version: 1,
            nodes: Vec::new(),
            frames: Vec::new(),
            materials: Vec::new(),
            triangles: Vec::new(),
        }
    }
}

struct Node {
    id: isize,
    name: String,
    parent: isize,
}

impl Node {
    fn new(id: isize, name: String, parent: isize) -> Self {
        Self { id, name, parent }
    }
}

struct Frame {
    animated_nodes: Vec<AnimationNode>,
}

impl Frame {
    fn new() -> Self {
        Self { animated_nodes: Vec::new() }
    }
}

struct AnimationNode {
    id: isize,
    position: Vector3,
    rotation: Angles,
}

impl AnimationNode {
    fn new(id: isize, position: Vector3, rotation: Angles) -> Self {
        Self { id, position, rotation }
    }
}

struct Triangle {
    material_index: usize,
    vertices: Vec<Vertex>,
}

impl Triangle {
    fn new(material_index: usize) -> Self {
        Self {
            material_index,
            vertices: Vec::with_capacity(3),
        }
    }
}

struct Vertex {
    position: Vector3,
    normal: Vector3,
    uv: Vector2,
    weights: Vec<(isize, f64)>,
    extra_uv: Vec<Vector2>,
}

impl Vertex {
    fn new(position: Vector3, normal: Vector3, uv: Vector2) -> Self {
        Self {
            position,
            normal,
            uv,
            weights: Vec::with_capacity(1),
            extra_uv: Vec::new(),
        }
    }
}
