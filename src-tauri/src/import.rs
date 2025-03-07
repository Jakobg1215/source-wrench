use std::{
    io::Error,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use indexmap::IndexMap;
use serde::Serialize;
use thiserror::Error as ThisError;

use crate::utilities::{
    logging::{log, LogLevel},
    mathematics::{Quaternion, Vector2, Vector3},
};

mod obj;
mod smd;

use obj::ParseOBJError;
use smd::ParseSMDError;

/// The collection of all data from a source file.
#[derive(Debug, Default, Serialize)]
pub struct ImportFileData {
    /// All the bones in the source file, mapped to their name.
    pub skeleton: IndexMap<String, ImportBone>,
    /// All the animations in the source file, mapped to their name.
    pub animations: IndexMap<String, ImportAnimation>,
    /// All the mesh parts in the source file, mapped to their name.
    pub parts: IndexMap<String, ImportPart>,
}

/// Data of a bone from a source file.
#[derive(Debug, Default, Serialize)]
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
#[derive(Debug, Serialize)]
pub struct ImportAnimation {
    /// The amount of frames the animation stores.
    pub frame_count: NonZeroUsize,
    /// Bones that are animated in the animation.
    pub channels: IndexMap<usize, ImportChannel>,
}

/// Data of an animated bone from a source file.
#[derive(Debug, Default, Serialize)]
pub struct ImportChannel {
    /// Positional keyed data of the channel, mapped to a frame.
    pub position: IndexMap<usize, Vector3>,
    /// Rotational keyed data of the channel, mapped to a frame.
    pub rotation: IndexMap<usize, Quaternion>,
}

/// Data of a mesh part from a source file.
#[derive(Debug, Default, Serialize)]
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
#[derive(Debug, Default, Serialize)]
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
#[derive(Debug, Default, Serialize)]
pub struct ImportFlexVertex {
    /// The new position of the vertex for the flex key.
    pub position: Vector3,
    /// The new normal direction of the vertex for the flex key.
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

#[derive(Debug, Default)]
pub struct FileManager {
    pub files: Mutex<IndexMap<PathBuf, Arc<ImportFileData>>>,
}

impl FileManager {
    pub fn load_file(&self, path: String) -> Result<Arc<ImportFileData>, ParseError> {
        let file_path = PathBuf::from(path);
        let mut files = self.files.lock().unwrap();

        if let Some(file) = files.get(&file_path) {
            return Ok(Arc::clone(file));
        }

        if !file_path.try_exists()? {
            return Err(ParseError::FileDoesNotExist);
        }

        let file_extension = file_path.extension().ok_or_else(|| ParseError::FileDoesNotHaveExtension)?;

        let imported_file = match file_extension.to_string_lossy().to_lowercase().as_str() {
            "smd" => smd::load_smd(&file_path)?,
            "obj" => obj::load_obj(&file_path)?,
            _ => return Err(ParseError::UnsupportedFileFormat),
        };

        debug_assert!(!imported_file.skeleton.is_empty(), "File source must have 1 bone!");
        debug_assert!(!imported_file.animations.is_empty(), "File source must have 1 animation!");

        log(
            format!(
                "Loaded {} file: {}",
                file_extension.to_string_lossy().to_uppercase(),
                file_path.as_os_str().to_string_lossy()
            ),
            LogLevel::Verbose,
        );
        let file = Arc::new(imported_file);
        files.insert(file_path, Arc::clone(&file));
        Ok(file)
    }

    pub fn unload_file(&self, path: String) {
        let file_path = PathBuf::from(path);
        let mut files = self.files.lock().unwrap();
        files.swap_remove(&file_path);
    }

    pub fn get_file(&self, path: &str) -> Option<Arc<ImportFileData>> {
        let file_path = Path::new(path);
        self.files.lock().unwrap().get(file_path).cloned()
    }

    pub fn loaded_file_count(&self) -> usize {
        self.files.lock().unwrap().len()
    }
}
