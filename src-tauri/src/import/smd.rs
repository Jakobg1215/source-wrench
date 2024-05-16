use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{BufRead, BufReader, Error},
    path::Path,
};

use thiserror::Error;

use crate::{
    import::ImportedBone,
    utilities::{
        logging::{log, LogLevel},
        mathematics::{Angles, Vector2, Vector3},
    },
};

use super::{ImportedBoneAnimation, ImportedFileData, ImportedVertex};

#[derive(Error, Debug)]
pub enum ParseSMDError {
    #[error("Failed To Open File")]
    FailedFileOpen(#[from] Error),
    #[error("Failed To Parse Integer On Line {0}")]
    FailedIntegerParse(usize),
    #[error("Failed To Parse Float On Line {0}")]
    FailedFloatParse(usize),
    #[error("Unexpected End Of File")]
    EndOFFile,
    #[error("Unknown Studio Command {0} On Line {1}")]
    UnknownStudioCommand(String, usize),
    #[error("Missing {0} Argument On Line {1}")]
    MissingArgument(&'static str, usize),
    #[error("Invalid SMD Version")]
    InvalidVersion,
    #[error("Node ID Is Not Sequential On Line {0}")]
    NonSequentialNode(usize),
    #[error("Invalid Node Index On Line {0}")]
    InvalidNodeIndex(usize),
    #[error("Frames Are Not Sequential On Line {0}")]
    NonSequentialFrames(usize),
    #[error("No Fame Specified Before Nodes On Line {0}")]
    NoFrame(usize),
    #[error("No Frames In File")]
    NoBindFrame,
    #[error("Not All Bones Specified")]
    MissingBoneBind,
}

#[derive(Default)]
struct SMDData {
    nodes: Vec<Node>,
    frames: Vec<Vec<Bone>>,
    vertices: Vec<Vertex>,
    materials: HashMap<String, Vec<Vec<usize>>>,
}

impl SMDData {
    fn add_vertex(&mut self, vertex: Vertex) -> usize {
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }
}

struct Node {
    name: String,
    parent: Option<usize>,
}

impl Node {
    fn new(name: String, parent: Option<usize>) -> Self {
        Self { name, parent }
    }
}

struct Bone {
    node: usize,
    position: Vector3,
    rotation: Angles,
}

impl Bone {
    fn new(node: usize, position: Vector3, rotation: Angles) -> Self {
        Self { node, position, rotation }
    }
}
struct Vertex {
    position: Vector3,
    normal: Vector3,
    texture_coordinate: Vector2,
    links: Vec<Link>,
}

impl Vertex {
    fn new(position: Vector3, normal: Vector3, texture_coordinate: Vector2) -> Self {
        Self {
            position,
            normal,
            texture_coordinate,
            links: Vec::with_capacity(1),
        }
    }
}

struct Link {
    bone: usize,
    weight: f64,
}

impl Link {
    fn new(bone: usize, weight: f64) -> Self {
        Self { bone, weight }
    }
}

pub fn load_smd(file_path: &Path, _vta_path: Option<&Path>) -> Result<ImportedFileData, ParseSMDError> {
    log(
        format!("Loading SMD File: {:?}", file_path.file_name().expect("File Path To Be Validated!")),
        LogLevel::Verbose,
    );

    let file = File::open(file_path)?;
    let file_buffer = BufReader::new(file);
    let mut lines = file_buffer.lines().flatten();
    let mut line_count = 0;
    let mut smd_data = SMDData::default();

    loop {
        let current_line = match lines.next() {
            Some(line) => line,
            None => break,
        };
        line_count += 1;

        let mut line_arguments = current_line.split_whitespace();
        let command = line_arguments.next();

        match command {
            Some("version") => {
                let version = match line_arguments.next() {
                    Some(version) => match version.parse::<isize>() {
                        Ok(version) => version,
                        Err(_) => return Err(ParseSMDError::FailedIntegerParse(line_count)),
                    },
                    None => return Err(ParseSMDError::MissingArgument("Version", line_count)),
                };

                if version < 1 || version > 2 {
                    return Err(ParseSMDError::InvalidVersion);
                }
            }
            Some("nodes") => loop {
                let current_line = match lines.next() {
                    Some(line) => line,
                    None => return Err(ParseSMDError::EndOFFile),
                };
                line_count += 1;

                struct SplitAtWhitespace<'a> {
                    input: &'a str,
                    index: usize,
                    in_quotes: bool,
                }

                impl<'a> SplitAtWhitespace<'a> {
                    fn new(input: &'a str) -> Self {
                        Self {
                            input,
                            index: 0,
                            in_quotes: false,
                        }
                    }
                }

                impl<'a> Iterator for SplitAtWhitespace<'a> {
                    type Item = &'a str;

                    fn next(&mut self) -> Option<Self::Item> {
                        let mut start = None;
                        let mut end = None;
                        let mut chars = self.input[self.index..].char_indices();

                        while let Some((index, char)) = chars.next() {
                            match char {
                                '"' => {
                                    self.in_quotes = !self.in_quotes;
                                    if self.in_quotes {
                                        start.get_or_insert(self.index + index + 1);
                                    } else if start.is_some() {
                                        end = Some(self.index + index);
                                        break;
                                    }
                                }
                                _ if char.is_whitespace() && !self.in_quotes => {
                                    if let Some(start) = start {
                                        end = Some(start + index);
                                        break;
                                    }
                                }
                                _ => {
                                    start.get_or_insert(self.index);
                                }
                            }
                        }

                        if let Some(start) = start {
                            let end = end.unwrap_or(self.input.len());
                            let word = &self.input[start..end];
                            self.index = end;
                            Some(word.trim())
                        } else {
                            None
                        }
                    }
                }

                let mut line_arguments = SplitAtWhitespace::new(&current_line);

                let index = match line_arguments.next() {
                    Some(index) => {
                        if index == "end" {
                            break;
                        }
                        match index.parse::<usize>() {
                            Ok(index) => index,
                            Err(_) => return Err(ParseSMDError::FailedIntegerParse(line_count)),
                        }
                    }
                    None => return Err(ParseSMDError::MissingArgument("index", line_count)),
                };

                if index != smd_data.nodes.len() {
                    return Err(ParseSMDError::NonSequentialNode(line_count));
                }

                let name = match line_arguments.next() {
                    Some(name) => name.to_string(),
                    None => return Err(ParseSMDError::MissingArgument("name", line_count)),
                };

                let parent = match line_arguments.next() {
                    Some(parent) => match parent.parse::<isize>() {
                        Ok(parent) => parent,
                        Err(_) => return Err(ParseSMDError::FailedIntegerParse(line_count)),
                    },
                    None => return Err(ParseSMDError::MissingArgument("parent", line_count)),
                };

                if parent < -1 || parent != -1 && parent as usize > smd_data.nodes.len() {
                    return Err(ParseSMDError::InvalidNodeIndex(line_count));
                }

                smd_data.nodes.push(Node::new(name, if parent == -1 { None } else { Some(parent as usize) }))
            },
            Some("skeleton") => loop {
                let current_line = match lines.next() {
                    Some(line) => line,
                    None => return Err(ParseSMDError::EndOFFile),
                };
                line_count += 1;

                let mut line_arguments = current_line.split_whitespace();

                let node = match line_arguments.next() {
                    Some(node) => match node {
                        "end" => break,
                        "time" => {
                            let time = match line_arguments.next() {
                                Some(time) => match time.parse::<usize>() {
                                    Ok(time) => time,
                                    Err(_) => return Err(ParseSMDError::FailedIntegerParse(line_count)),
                                },
                                None => return Err(ParseSMDError::MissingArgument("frame", line_count)),
                            };

                            if time != smd_data.frames.len() {
                                return Err(ParseSMDError::NonSequentialFrames(line_count));
                            }

                            smd_data.frames.push(Vec::new());
                            continue;
                        }
                        _ => match node.parse::<usize>() {
                            Ok(node) => node,
                            Err(_) => return Err(ParseSMDError::FailedIntegerParse(line_count)),
                        },
                    },

                    None => return Err(ParseSMDError::MissingArgument("node", line_count)),
                };

                let frame_count = smd_data.frames.len();

                let frame = match smd_data.frames.last_mut() {
                    Some(frame) => frame,
                    None => return Err(ParseSMDError::NoFrame(line_count)),
                };

                if node > smd_data.nodes.len() {
                    return Err(ParseSMDError::InvalidNodeIndex(line_count));
                }

                if frame_count == 1 && node != frame.len() {
                    return Err(ParseSMDError::NonSequentialNode(line_count));
                }

                let x_position = match line_arguments.next() {
                    Some(x_position) => match x_position.parse::<f64>() {
                        Ok(x_position) => x_position,
                        Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                    },
                    None => return Err(ParseSMDError::MissingArgument("X Position", line_count)),
                };

                let y_position = match line_arguments.next() {
                    Some(y_position) => match y_position.parse::<f64>() {
                        Ok(y_position) => y_position,
                        Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                    },
                    None => return Err(ParseSMDError::MissingArgument("Y Position", line_count)),
                };

                let z_position = match line_arguments.next() {
                    Some(z_position) => match z_position.parse::<f64>() {
                        Ok(z_position) => z_position,
                        Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                    },
                    None => return Err(ParseSMDError::MissingArgument("Z Position", line_count)),
                };

                let x_rotation = match line_arguments.next() {
                    Some(x_rotation) => match x_rotation.parse::<f64>() {
                        Ok(x_rotation) => x_rotation,
                        Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                    },
                    None => return Err(ParseSMDError::MissingArgument("X Rotation", line_count)),
                };

                let y_rotation = match line_arguments.next() {
                    Some(y_rotation) => match y_rotation.parse::<f64>() {
                        Ok(y_rotation) => y_rotation,
                        Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                    },
                    None => return Err(ParseSMDError::MissingArgument("Y Rotation", line_count)),
                };

                let z_rotation = match line_arguments.next() {
                    Some(z_rotation) => match z_rotation.parse::<f64>() {
                        Ok(z_rotation) => z_rotation,
                        Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                    },
                    None => return Err(ParseSMDError::MissingArgument("Z Rotation", line_count)),
                };

                frame.push(Bone::new(
                    node,
                    Vector3::new(x_position, y_position, z_position),
                    Angles::new(x_rotation, y_rotation, z_rotation),
                ));
            },
            Some("triangles") => loop {
                let current_line = match lines.next() {
                    Some(line) => line,
                    None => return Err(ParseSMDError::EndOFFile),
                };
                line_count += 1;

                if current_line == "end" {
                    break;
                }

                let mut face = Vec::with_capacity(3);

                for _ in 0..3 {
                    let current_line = match lines.next() {
                        Some(line) => line,
                        None => return Err(ParseSMDError::EndOFFile),
                    };
                    line_count += 1;

                    let mut line_arguments = current_line.split_whitespace();

                    let bone = match line_arguments.next() {
                        Some(bone) => match bone.parse::<usize>() {
                            Ok(bone) => bone,
                            Err(_) => return Err(ParseSMDError::FailedIntegerParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("Bone", line_count)),
                    };

                    if bone > smd_data.nodes.len() {
                        return Err(ParseSMDError::InvalidNodeIndex(line_count));
                    }

                    let x_position = match line_arguments.next() {
                        Some(x_position) => match x_position.parse::<f64>() {
                            Ok(x_position) => x_position,
                            Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("X Position", line_count)),
                    };

                    let y_position = match line_arguments.next() {
                        Some(y_position) => match y_position.parse::<f64>() {
                            Ok(y_position) => y_position,
                            Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("Y Position", line_count)),
                    };

                    let z_position = match line_arguments.next() {
                        Some(z_position) => match z_position.parse::<f64>() {
                            Ok(z_position) => z_position,
                            Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("Z Position", line_count)),
                    };

                    let x_normal = match line_arguments.next() {
                        Some(x_normal) => match x_normal.parse::<f64>() {
                            Ok(x_normal) => x_normal,
                            Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("X Normal", line_count)),
                    };

                    let y_normal = match line_arguments.next() {
                        Some(y_normal) => match y_normal.parse::<f64>() {
                            Ok(y_normal) => y_normal,
                            Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("Y Normal", line_count)),
                    };

                    let z_normal = match line_arguments.next() {
                        Some(z_normal) => match z_normal.parse::<f64>() {
                            Ok(z_normal) => z_normal,
                            Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("Z Normal", line_count)),
                    };

                    let u_texture_coordinate = match line_arguments.next() {
                        Some(u_texture_coordinate) => match u_texture_coordinate.parse::<f64>() {
                            Ok(u_texture_coordinate) => u_texture_coordinate,
                            Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("U Texture Coordinate", line_count)),
                    };

                    let v_texture_coordinate = match line_arguments.next() {
                        Some(v_texture_coordinate) => match v_texture_coordinate.parse::<f64>() {
                            Ok(v_texture_coordinate) => v_texture_coordinate,
                            Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                        },
                        None => return Err(ParseSMDError::MissingArgument("V Texture Coordinate", line_count)),
                    };

                    let link_count = match line_arguments.next() {
                        Some(link_count) => match link_count.parse::<usize>() {
                            Ok(link_count) => link_count,
                            Err(_) => return Err(ParseSMDError::FailedIntegerParse(line_count)),
                        },
                        None => 0,
                    };

                    let mut vertex = Vertex::new(
                        Vector3::new(x_position, y_position, z_position),
                        Vector3::new(x_normal, y_normal, z_normal),
                        Vector2::new(u_texture_coordinate, v_texture_coordinate),
                    );

                    if link_count == 0 {
                        vertex.links.push(Link::new(bone, 1.0));
                        let vertex_index = smd_data.add_vertex(vertex);
                        face.push(vertex_index);
                        continue;
                    }

                    vertex.links.reserve(link_count);

                    for _ in 0..link_count {
                        let bone = match line_arguments.next() {
                            Some(bone) => match bone.parse::<usize>() {
                                Ok(bone) => bone,
                                Err(_) => return Err(ParseSMDError::FailedIntegerParse(line_count)),
                            },
                            None => return Err(ParseSMDError::MissingArgument("Bone Link", line_count)),
                        };

                        let weight = match line_arguments.next() {
                            Some(weight) => match weight.parse::<f64>() {
                                Ok(weight) => weight,
                                Err(_) => return Err(ParseSMDError::FailedFloatParse(line_count)),
                            },
                            None => return Err(ParseSMDError::MissingArgument("Weight Link", line_count)),
                        };

                        if bone > smd_data.nodes.len() {
                            return Err(ParseSMDError::InvalidNodeIndex(line_count));
                        }

                        vertex.links.push(Link::new(bone, weight))
                    }

                    let vertex_index = smd_data.add_vertex(vertex);
                    face.push(vertex_index);
                }

                let face_list = match smd_data.materials.entry(current_line) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => entry.insert(Vec::new()),
                };

                face_list.push(face);
            },
            Some(command) => return Err(ParseSMDError::UnknownStudioCommand(command.to_string(), line_count)),
            None => continue,
        }
    }

    let mut file_data = ImportedFileData::default();

    if smd_data.frames.len() == 0 {
        return Err(ParseSMDError::NoBindFrame);
    }

    if smd_data.frames[0].len() != smd_data.nodes.len() {
        return Err(ParseSMDError::MissingBoneBind);
    }

    let mut mapped_nodes = HashMap::new();

    for (id, node) in smd_data.nodes.into_iter().enumerate() {
        let bind_frame = &smd_data.frames[0];

        let bind_pose = match bind_frame.iter().find(|bone| bone.node == id) {
            Some(bind_pose) => bind_pose,
            None => return Err(ParseSMDError::MissingBoneBind),
        };

        let parent = match node.parent {
            Some(parent) => match mapped_nodes.get(&parent) {
                Some(parent) => Some(*parent),
                None => None,
            },
            None => None,
        };

        let index = file_data.add_bone(ImportedBone::new(node.name, bind_pose.position, bind_pose.rotation.to_quaternion(), parent));
        mapped_nodes.insert(id, index);
    }

    for (frame, keys) in smd_data.frames.into_iter().enumerate() {
        for bone in keys {
            let index = mapped_nodes.get(&bone.node).expect("Node Not Found!");
            let animation = file_data.animation.get_mut(*index).expect("Animation Not Found!");
            animation.insert(frame, ImportedBoneAnimation::new(bone.position, bone.rotation.to_quaternion()));
        }
    }

    file_data.mesh.materials = smd_data.materials;

    for vertex in smd_data.vertices {
        let mut vert = ImportedVertex::new(vertex.position, vertex.normal, vertex.texture_coordinate);
        for link in vertex.links {
            let bone_index = mapped_nodes.get(&link.bone).expect("Bone Not Found!");
            vert.add_weight(*bone_index, link.weight)
        }
        file_data.mesh.add_vertex(vert);
    }

    Ok(file_data)
}
