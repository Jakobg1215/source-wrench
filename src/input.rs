use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct InputCompilationData {
    /// The name of the mdl output as *.mdl
    pub model_name: String,
    /// The path to where the mdl is exported.
    pub export_path: Option<PathBuf>,
    pub body_groups: Vec<InputBodyPart>,
    pub animations: Vec<InputAnimation>,
    pub sequences: Vec<InputSequence>,
}

/// A struct to define a body part for the model.
#[derive(Clone, Debug)]
pub struct InputBodyPart {
    pub name: String,
    /// The models used by the body part.
    pub models: Vec<InputModel>,
}

impl Default for InputBodyPart {
    fn default() -> Self {
        Self {
            name: String::from("New Body Group"),
            models: Default::default(),
        }
    }
}

/// A struct to define a model for a body part.
#[derive(Clone, Debug)]
pub struct InputModel {
    pub name: String,
    /// This specify if the model will have no mesh.
    pub blank: bool,
    /// The source file to get the mesh data from.
    pub source_file_path: Option<PathBuf>,
    /// All the parts to use in the source file.
    pub enabled_source_parts: Vec<bool>,
}

impl Default for InputModel {
    fn default() -> Self {
        Self {
            name: String::from("New Model"),
            blank: Default::default(),
            source_file_path: Default::default(),
            enabled_source_parts: Default::default(),
        }
    }
}

/// A struct to define an animation for the model.
#[derive(Clone, Debug)]
pub struct InputAnimation {
    pub name: String,
    /// The source file to get the animation data from.
    pub source_file_path: Option<PathBuf>,
    /// The animation to get in the source file.
    pub source_animation: usize,
}

impl Default for InputAnimation {
    fn default() -> Self {
        Self {
            name: String::from("New Animation"),
            source_file_path: Default::default(),
            source_animation: Default::default(),
        }
    }
}

/// A struct the define a sequence for a model.
#[derive(Clone, Debug)]
pub struct InputSequence {
    pub name: String,
    /// A N by N grid of animations used by the sequence.
    pub animations: Vec<Vec<usize>>,
}

impl Default for InputSequence {
    fn default() -> Self {
        Self {
            name: String::from("New Sequence"),
            animations: Default::default(),
        }
    }
}
