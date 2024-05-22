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
    pub is_blank: bool,
    pub model_source: String,
}

#[derive(Deserialize)]
pub struct ImputedAnimation {
    pub name: String,
    pub source_file: String,
}

#[derive(Deserialize)]
pub struct ImputedSequence {
    pub name: String,
    pub animation: String,
}
