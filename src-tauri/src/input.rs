use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImputedCompilationData {
    pub model_name: String,
    pub body_parts: Vec<ImputedBodyPart>,
    pub animations: Vec<ImputedAnimation>,
    pub sequences: Vec<ImputedSequence>,
    pub export_path: String,
}

#[derive(Deserialize)]
pub struct ImputedBodyPart {
    pub name: String,
    pub models: Vec<ImputedModel>,
}

#[derive(Deserialize)]
pub struct ImputedModel {
    pub name: String,
    pub model_source: String,
    pub part_name: Vec<String>,
}

#[derive(Deserialize)]
pub struct ImputedAnimation {
    pub name: String,
    pub source_file: String,
    pub animation_name: String,
}

#[derive(Deserialize)]
pub struct ImputedSequence {
    pub name: String,
    pub animation: String,
}
