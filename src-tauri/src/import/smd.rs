use std::{
    fs::File,
    io::{BufRead, BufReader},
    num::NonZero,
    path::Path,
};

use indexmap::IndexMap;
use thiserror::Error as ThisError;

use crate::utilities::mathematics::{Angles, Vector2, Vector3};

use super::{ImportAnimation, ImportBone, ImportFileData, ImportFlexVertex, ImportPart, ImportVertex};

#[derive(Debug, ThisError)]
pub enum ParseSMDError {
    #[error("Unknown Command {0} On Line {1}")]
    UnknownCommand(String, usize),
    #[error("Missing {0} Argument On Line {1}")]
    MissingArgument(&'static str, usize),
    #[error("Failed To Parse Integer On Line {0}")]
    FailedIntegerParse(usize),
    #[error("Invalid Version On Line {0}")]
    InvalidVersion(usize),
    #[error("Node Index Is Not Sequential On Line {0}")]
    InvalidNodeIndex(usize),
    #[error("Invalid Node Parent Index On Line {0}")]
    InvalidNodeParentIndex(usize),
    #[error("Invalid Frame Index {0}")]
    InvalidFrameIndex(usize),
    #[error("File Ended Unexpectedly")]
    UnexpectedEndOfFile,
    #[error("No Frames In File")]
    NoBindFrame,
    #[error("Not All Bones Specified")]
    MissingBoneBind,
}

pub fn load_smd(file_path: &Path) -> Result<ImportFileData, ParseSMDError> {
    let file = File::open(file_path).expect("This should be checked before called!");
    let file_buffer = BufReader::new(file);
    let mut lines = file_buffer.lines().map_while(Result::ok);
    let mut line_count = 0;

    struct SplitAtWhitespace {
        input: String,
    }

    impl SplitAtWhitespace {
        fn new(input: String) -> Self {
            Self { input }
        }
    }

    impl Iterator for SplitAtWhitespace {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            if self.input.is_empty() {
                return None;
            }

            let characters = self.input.as_bytes();

            let mut output = None;
            let mut in_quotes = false;
            let mut comment_check = false;
            let mut in_comment = false;

            let mut index = 0;
            while index < self.input.len() {
                let character = characters[index] as char;
                index += 1;

                if in_comment {
                    continue;
                }

                if character.is_whitespace() {
                    if in_quotes || output.is_none() {
                        continue;
                    }

                    if !in_quotes && output.is_some() {
                        break;
                    }
                }

                match character {
                    '"' => {
                        if in_quotes {
                            break;
                        }

                        in_quotes = true;
                    }
                    '/' => {
                        if in_quotes {
                            output.get_or_insert_with(String::new).push(character);
                        }

                        if comment_check {
                            in_comment = true;
                            continue;
                        }

                        comment_check = true;
                    }
                    ';' | '#' => {
                        if in_quotes {
                            output.get_or_insert_with(String::new).push(character);
                            continue;
                        }

                        in_comment = true;
                    }
                    char => {
                        output.get_or_insert_with(String::new).push(char);
                        comment_check = false;
                    }
                }
            }

            self.input.drain(..index);

            output
        }
    }

    struct Node {
        name: String,
        parent: Option<usize>,
    }

    let mut nodes = Vec::new();

    struct KeyFrame {
        position: Vector3,
        rotation: Angles,
    }

    let mut frames = Vec::new();

    struct Vertex {
        position: Vector3,
        normal: Vector3,
        texture_coordinate: Vector2,
        links: IndexMap<usize, f64>,
    }

    let mut triangles: IndexMap<String, Vec<[Vertex; 3]>> = IndexMap::new();

    struct FlexVertex {
        position: Vector3,
        normal: Vector3,
    }

    let mut flexes = Vec::new();

    while let Some(line) = lines.next() {
        line_count += 1;

        let mut line_arguments = SplitAtWhitespace::new(line);
        let command = match line_arguments.next() {
            Some(command) => command,
            None => continue,
        };

        match command.as_str() {
            "version" => {
                let version = line_arguments
                    .next()
                    .ok_or(ParseSMDError::MissingArgument("Version", line_count))?
                    .parse::<isize>()
                    .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?;

                if !(1..=2).contains(&version) {
                    return Err(ParseSMDError::InvalidVersion(line_count));
                }
            }
            "nodes" => {
                for line in lines.by_ref() {
                    line_count += 1;

                    if line.starts_with("end") {
                        break;
                    }

                    let mut line_arguments = SplitAtWhitespace::new(line);

                    let node_index = match line_arguments.next() {
                        Some(index) => index.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        None => continue,
                    };

                    if node_index != nodes.len() {
                        return Err(ParseSMDError::InvalidNodeIndex(line_count));
                    }

                    let node_name = line_arguments.next().ok_or(ParseSMDError::MissingArgument("Node Name", line_count))?;

                    let node_parent = match line_arguments.next() {
                        Some(index) => {
                            if index.starts_with('-') {
                                None
                            } else {
                                Some(index.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?)
                            }
                        }
                        None => return Err(ParseSMDError::MissingArgument("Node Parent", line_count)),
                    };

                    if node_parent.is_some() && node_parent.unwrap() > nodes.len() {
                        return Err(ParseSMDError::InvalidNodeParentIndex(line_count));
                    }

                    nodes.push(Node {
                        name: node_name,
                        parent: node_parent,
                    });
                }
            }
            "skeleton" => {
                for line in lines.by_ref() {
                    line_count += 1;

                    if line.starts_with("end") {
                        break;
                    }

                    let mut line_arguments = SplitAtWhitespace::new(line);

                    let node_index = match line_arguments.next() {
                        Some(index) => {
                            if index.starts_with("time") {
                                match line_arguments.next() {
                                    Some(time) => {
                                        let time = time.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?;
                                        if time != frames.len() {
                                            return Err(ParseSMDError::InvalidFrameIndex(line_count));
                                        }
                                        frames.push(IndexMap::new());
                                        continue;
                                    }
                                    None => return Err(ParseSMDError::MissingArgument("Time", line_count)),
                                }
                            }

                            index.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?
                        }
                        None => continue,
                    };

                    if node_index > nodes.len() {
                        return Err(ParseSMDError::InvalidNodeIndex(line_count));
                    }

                    let position = Vector3::new(
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Position X", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Position Y", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Position Z", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                    );

                    let rotation = Angles::new(
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Rotation X", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Rotation Y", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Rotation Z", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                    );

                    let previous_frame = frames.last_mut().unwrap();
                    previous_frame.insert(node_index, KeyFrame { position, rotation });
                }
            }
            "triangles" => {
                while let Some(line) = lines.next() {
                    line_count += 1;

                    if line.starts_with("end") {
                        break;
                    }

                    let mut line_arguments = SplitAtWhitespace::new(line);

                    let material = match line_arguments.next() {
                        Some(material) => material,
                        None => continue,
                    };

                    fn parse_vertex(lines: &mut impl Iterator<Item = String>, line_count: &mut usize, node_count: usize) -> Result<Vertex, ParseSMDError> {
                        for line in lines {
                            *line_count += 1;

                            let mut line_arguments = SplitAtWhitespace::new(line);

                            let node_index = match line_arguments.next() {
                                Some(index) => index.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                None => continue,
                            };

                            if node_index > node_count {
                                return Err(ParseSMDError::InvalidNodeIndex(*line_count));
                            }

                            let position = Vector3::new(
                                line_arguments
                                    .next()
                                    .ok_or(ParseSMDError::MissingArgument("Position X", *line_count))?
                                    .parse::<f64>()
                                    .map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                line_arguments
                                    .next()
                                    .ok_or(ParseSMDError::MissingArgument("Position Y", *line_count))?
                                    .parse::<f64>()
                                    .map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                line_arguments
                                    .next()
                                    .ok_or(ParseSMDError::MissingArgument("Position Z", *line_count))?
                                    .parse::<f64>()
                                    .map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                            );

                            let normal = Vector3::new(
                                line_arguments
                                    .next()
                                    .ok_or(ParseSMDError::MissingArgument("Normal X", *line_count))?
                                    .parse::<f64>()
                                    .map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                line_arguments
                                    .next()
                                    .ok_or(ParseSMDError::MissingArgument("Normal Y", *line_count))?
                                    .parse::<f64>()
                                    .map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                line_arguments
                                    .next()
                                    .ok_or(ParseSMDError::MissingArgument("Normal Z", *line_count))?
                                    .parse::<f64>()
                                    .map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                            );

                            let texture_coordinate = Vector2::new(
                                line_arguments
                                    .next()
                                    .ok_or(ParseSMDError::MissingArgument("Texture Coordinate X", *line_count))?
                                    .parse::<f64>()
                                    .map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                line_arguments
                                    .next()
                                    .ok_or(ParseSMDError::MissingArgument("Texture Coordinate Y", *line_count))?
                                    .parse::<f64>()
                                    .map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                            );

                            let mut vertex = Vertex {
                                position,
                                normal,
                                texture_coordinate,
                                links: IndexMap::new(),
                            };

                            let link_count = match line_arguments.next() {
                                Some(count) => count.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                None => {
                                    vertex.links.insert(node_index, 1.0);
                                    return Ok(vertex);
                                }
                            };

                            if link_count == 0 {
                                vertex.links.insert(node_index, 1.0);
                                return Ok(vertex);
                            }

                            for _ in 0..link_count {
                                let node_index = match line_arguments.next() {
                                    Some(index) => index.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                    None => return Err(ParseSMDError::MissingArgument("Link Node", *line_count)),
                                };

                                if node_index > node_count {
                                    return Err(ParseSMDError::InvalidNodeIndex(*line_count));
                                }

                                let weight = match line_arguments.next() {
                                    Some(weight) => weight.parse::<f64>().map_err(|_| ParseSMDError::FailedIntegerParse(*line_count))?,
                                    None => return Err(ParseSMDError::MissingArgument("Link Weight", *line_count)),
                                };

                                vertex.links.insert(node_index, weight);
                            }

                            return Ok(vertex);
                        }
                        Err(ParseSMDError::UnexpectedEndOfFile)
                    }

                    let triangle = [
                        parse_vertex(lines.by_ref(), &mut line_count, nodes.len())?,
                        parse_vertex(lines.by_ref(), &mut line_count, nodes.len())?,
                        parse_vertex(lines.by_ref(), &mut line_count, nodes.len())?,
                    ];

                    let triangle_list = triangles.entry(material).or_default();
                    triangle_list.push(triangle);
                }
            }
            "vertexanimation" => {
                for line in lines.by_ref() {
                    line_count += 1;

                    if line.starts_with("end") {
                        break;
                    }

                    let mut line_arguments = SplitAtWhitespace::new(line);

                    let vertex_index = match line_arguments.next() {
                        Some(index) => {
                            if index.starts_with("time") {
                                match line_arguments.next() {
                                    Some(time) => {
                                        let time = time.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?;
                                        if time != frames.len() {
                                            return Err(ParseSMDError::InvalidFrameIndex(line_count));
                                        }
                                        flexes.push(IndexMap::new());
                                        continue;
                                    }
                                    None => return Err(ParseSMDError::MissingArgument("Time", line_count)),
                                }
                            }

                            index.parse::<usize>().map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?
                        }
                        None => continue,
                    };

                    let position = Vector3::new(
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Position X", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Position Y", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Position Z", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                    );

                    let normal = Vector3::new(
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Normal X", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Normal Y", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                        line_arguments
                            .next()
                            .ok_or(ParseSMDError::MissingArgument("Normal Z", line_count))?
                            .parse::<f64>()
                            .map_err(|_| ParseSMDError::FailedIntegerParse(line_count))?,
                    );

                    let previous_flex = flexes.last_mut().unwrap();
                    previous_flex.insert(vertex_index, FlexVertex { position, normal });
                }
            }
            _ => return Err(ParseSMDError::UnknownCommand(command, line_count)),
        }
    }

    if !flexes.is_empty() {
        let mut flex_part = ImportPart::default();

        for (frame, flex) in flexes.into_iter().enumerate() {
            let mut import_flex = IndexMap::with_capacity(flex.len());

            for (vertex_index, flex_data) in flex {
                import_flex.insert(
                    vertex_index,
                    ImportFlexVertex {
                        position: flex_data.position,
                        normal: flex_data.normal,
                    },
                );
            }

            flex_part.flexes.insert(format!("frame{}", frame), import_flex);
        }

        return Ok(ImportFileData {
            parts: IndexMap::from([(file_path.file_stem().unwrap().to_string_lossy().to_string(), flex_part)]),
            ..Default::default()
        });
    }

    if frames.is_empty() {
        return Err(ParseSMDError::NoBindFrame);
    }

    if frames[0].len() != nodes.len() {
        return Err(ParseSMDError::MissingBoneBind);
    }

    let bind_frame = &frames[0];
    let mut import_bones = IndexMap::with_capacity(nodes.len());
    for (id, node) in nodes.into_iter().enumerate() {
        let bind_pose = bind_frame.get(&id).unwrap();

        import_bones.insert(
            node.name,
            ImportBone {
                parent: node.parent,
                position: bind_pose.position,
                orientation: bind_pose.rotation.to_quaternion(),
            },
        );
    }

    let mut animation = ImportAnimation {
        frame_count: NonZero::new(frames.len()).unwrap(),
        channels: IndexMap::with_capacity(import_bones.len()),
    };

    for (frame, keys) in frames.into_iter().enumerate() {
        for (bone, key) in keys {
            let channel = animation.channels.entry(bone).or_default();
            channel.position.insert(frame, key.position);
            channel.rotation.insert(frame, key.rotation.to_quaternion());
        }
    }
    animation.channels.sort_keys();

    let mut parts = IndexMap::new();

    if !triangles.is_empty() {
        let mut part = ImportPart::default();

        for (material, vertices) in triangles {
            let polygon_list = part.polygons.entry(material).or_default();

            for triangle in vertices {
                let mut polygon = Vec::with_capacity(3);
                for vertex in triangle {
                    polygon.push(part.vertices.len());

                    part.vertices.push(ImportVertex {
                        position: vertex.position,
                        normal: vertex.normal,
                        texture_coordinate: vertex.texture_coordinate,
                        links: vertex.links,
                    });
                }
                polygon_list.push(polygon);
            }
        }

        parts.insert(file_path.file_stem().unwrap().to_string_lossy().to_string(), part);
    }

    Ok(ImportFileData {
        skeleton: import_bones,
        animations: IndexMap::from_iter([(file_path.file_stem().unwrap().to_string_lossy().to_string(), animation)]),
        parts,
    })
}
