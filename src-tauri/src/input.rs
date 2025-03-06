use indexmap::IndexMap;
use serde::Deserialize;

/// The main struct from the ui to specify and edit data for the compiler.
#[derive(Debug, Deserialize)]
pub struct ImputedCompilationData {
    /// The name of the mdl output as *.mdl
    pub model_name: String,
    /// The path to where the mdl is exported.
    pub export_path: String,
    /// List of body parts with the name mapped to the data.
    pub body_parts: IndexMap<String, ImputedBodyPart>,
    pub animations: IndexMap<String, ImputedAnimation>,
    pub sequences: IndexMap<String, ImputedSequence>,
}

/// A struct to define a body part for the model.
#[derive(Debug, Deserialize)]
pub struct ImputedBodyPart {
    /// The models used by the body part with the data mapped to the name.
    pub models: IndexMap<String, ImputedModel>,
}

/// A struct to define a model for a body part.
#[derive(Debug, Deserialize)]
pub struct ImputedModel {
    /// This specify if the model will have no mesh.
    pub is_blank: bool,
    /// The source file to get the mesh data from.
    pub file_source: String,
    /// All the parts to use in the source file.
    pub part_names: Vec<String>,
}

/// A struct to define an animation for the model.
#[derive(Debug, Deserialize)]
pub struct ImputedAnimation {
    /// The source file to get the animation data from.
    pub file_source: String,
    /// The animation to get in the source file.
    pub animation_name: String,
}

/// A struct the define a sequence for a model.
#[derive(Debug, Deserialize)]
pub struct ImputedSequence {
    /// A N by N grid of animations used by the sequence.
    pub animations: Vec<Vec<String>>,
}
