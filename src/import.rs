use std::{
    io::Error,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
};

use indexmap::IndexMap;
use thiserror::Error as ThisError;

use crate::utilities::{
    logging::{log, LogLevel},
    mathematics::{Quaternion, Vector2, Vector3},
};

mod obj;
mod smd;

use obj::ParseOBJError;
use smd::ParseSMDError;

pub const SUPPORTED_FILES: [&str; 2] = ["smd", "obj"];

/// The collection of all data from a source file.
#[derive(Debug, Default)]
pub struct ImportFileData {
    /// All the bones in the source file, mapped to their name.
    pub skeleton: IndexMap<String, ImportBone>,
    /// All the animations in the source file, mapped to their name.
    pub animations: IndexMap<String, ImportAnimation>,
    /// All the mesh parts in the source file, mapped to their name.
    pub parts: IndexMap<String, ImportPart>,
}

/// Data of a bone from a source file.
#[derive(Debug, Default)]
pub struct ImportBone {
    /// The index to the source file skeleton the bone is parented to.
    ///
    /// Is [`None`] when bone is a root bone.
    pub parent: Option<usize>,
    /// The position of the bone relative to the parent.
    ///
    /// If [`parent`][Self::parent] is [`None`] then position is absolute.
    pub position: Vector3,
    /// The orientation of the bone relative to the parent.
    ///
    /// If [`parent`][Self::parent] is [`None`] then orientation is absolute.
    pub orientation: Quaternion,
}

/// Data of an animation from a source file.
#[derive(Debug)]
pub struct ImportAnimation {
    /// The amount of frames the animation stores.
    pub frame_count: NonZeroUsize,
    /// Bones that are animated in the animation.
    pub channels: IndexMap<usize, ImportChannel>,
}

/// Data of an animated bone from a source file.
#[derive(Debug, Default)]
pub struct ImportChannel {
    /// Positional keyed data of the channel, mapped to a frame.
    pub position: IndexMap<usize, Vector3>,
    /// Rotational keyed data of the channel, mapped to a frame.
    pub rotation: IndexMap<usize, Quaternion>,
}

/// Data of a mesh part from a source file.
#[derive(Debug, Default)]
pub struct ImportPart {
    pub vertices: Vec<ImportVertex>,
    /// List of polygons the part has, mapped to the material name.
    ///
    /// A polygon is defined by an index list into [`vertices`][Self::vertices].
    pub polygons: IndexMap<String, Vec<Vec<usize>>>,
    /// List of flex data, mapped to their name.
    ///
    /// A flex stores a list of indexes that map into [`vertices`][Self::vertices] that are flexed.
    pub flexes: IndexMap<String, IndexMap<usize, ImportFlexVertex>>,
}

/// Data of a vertex from a source file.
#[derive(Debug, Default)]
pub struct ImportVertex {
    /// The position of the vertex, the position is absolute.
    pub position: Vector3,
    /// The normal direction of the vertex.
    pub normal: Vector3,
    /// The UV position of the vertex.
    pub texture_coordinate: Vector2,
    /// List of weights the vertex has, mapped to a bone by an index into [`skeleton`][ImportFileData::skeleton].
    pub links: IndexMap<usize, f64>,
}

/// Data of a flexed vertex data from a source file.
#[derive(Debug, Default)]
pub struct ImportFlexVertex {
    /// The new position of the vertex for the flex key.
    #[allow(dead_code)]
    pub position: Vector3,
    /// The new normal direction of the vertex for the flex key.
    #[allow(dead_code)]
    pub normal: Vector3,
}

#[derive(Debug, ThisError)]
pub enum ParseError {
    #[error("Failed To Open File")]
    FailedFileOpen(#[from] Error),
    #[error("File Does Not Exist")]
    FileDoesNotExist,
    #[error("File Does Not Have Extension")]
    FileDoesNotHaveExtension,
    #[error("File Format Is Not Supported")]
    UnsupportedFileFormat,
    // When supporting another file format, put it under this comment.
    #[error("Failed To Parse SMD File: {0}")]
    FailedSMDFileParse(#[from] ParseSMDError),
    #[error("Failed To Parse OBJ File: {0}")]
    FailedOBJFileParse(#[from] ParseOBJError),
}

#[derive(Debug, Clone)]
pub enum FileStatus {
    Loading,
    Loaded(Arc<ImportFileData>),
    Failed,
}

#[derive(Debug, Default)]
pub struct FileManager(Arc<RwLock<IndexMap<PathBuf, (usize, FileStatus)>>>);

impl FileManager {
    pub fn load_file(&mut self, file_path: PathBuf) {
        let mut files = self.0.write().unwrap();
        if let Some((existing_count, _)) = files.get_mut(&file_path) {
            *existing_count += 1;
            return;
        }
        files.insert(file_path.clone(), (1, FileStatus::Loading));
        drop(files);

        let manager = Arc::clone(&self.0);
        thread::spawn(move || {
            let loaded_file = (|| {
                if !file_path.try_exists()? {
                    return Err(ParseError::FileDoesNotExist);
                }

                let file_extension = file_path.extension().ok_or_else(|| ParseError::FileDoesNotHaveExtension)?;

                let loaded_file = match file_extension.to_string_lossy().to_lowercase().as_str() {
                    "smd" => smd::load_smd(&file_path)?,
                    "obj" => obj::load_obj(&file_path)?,
                    _ => return Err(ParseError::UnsupportedFileFormat),
                };

                log(
                    format!(
                        "Loaded {} file: {}",
                        file_extension.to_string_lossy().to_uppercase(),
                        file_path.as_os_str().to_string_lossy()
                    ),
                    LogLevel::Verbose,
                );

                Ok(loaded_file)
            })();

            let mut files = manager.write().unwrap();

            let file_data = match loaded_file {
                Ok(data) => data,
                Err(error) => {
                    log(format!("Fail To Load File: {}!", error), LogLevel::Error);

                    if let Some(entry) = files.get_mut(&file_path) {
                        entry.1 = FileStatus::Failed;
                    }

                    return;
                }
            };

            debug_assert!(!file_data.skeleton.is_empty(), "File source must have 1 bone!");
            debug_assert!(!file_data.animations.is_empty(), "File source must have 1 animation!");

            if let Some(entry) = files.get_mut(&file_path) {
                entry.1 = FileStatus::Loaded(Arc::new(file_data));
            }
        });
    }

    pub fn get_file_status(&self, file_path: &PathBuf) -> Option<FileStatus> {
        let files = self.0.read().unwrap();
        files.get(file_path).map(|(_, status)| status).cloned()
    }

    pub fn get_file_data(&self, file_path: &PathBuf) -> Option<Arc<ImportFileData>> {
        let files = self.0.read().unwrap();
        files
            .get(file_path)
            .and_then(|(_, status)| if let FileStatus::Loaded(data) = status { Some(data.clone()) } else { None })
    }

    pub fn unload_file(&mut self, file_path: &PathBuf) {
        let mut files = self.0.write().unwrap();
        if let Some((existing_count, _)) = files.get_mut(file_path) {
            let current_count = *existing_count - 1;

            if current_count == 0 {
                log(format!("Unloaded {}", file_path.as_os_str().to_string_lossy()), LogLevel::Debug);
                files.shift_remove(file_path);
                return;
            }

            *existing_count = current_count;
        }
    }

    pub fn loaded_file_count(&self) -> usize {
        let files = self.0.write().unwrap();
        files.len()
    }

    pub fn is_loading_files(&self) -> bool {
        let files = self.0.write().unwrap();
        files
            .values()
            .any(|(_, file_status)| matches!(file_status, FileStatus::Loading | FileStatus::Failed))
    }
}
