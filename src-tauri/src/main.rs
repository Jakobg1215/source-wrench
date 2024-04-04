// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Deserialize;

#[derive(Deserialize)]
struct CompilationData {
    model_name: String,
    body_groups: Vec<BodyGroup>,
}

#[derive(Deserialize)]
struct BodyGroup {
    name: String,
    parts: Vec<BodyPart>,
}

#[derive(Deserialize)]
struct BodyPart {
    name: String,
    is_blank: bool,
    model_source: String,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn compile_model(data: CompilationData) {
    println!("Compiling model {}", data.model_name);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![compile_model])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
