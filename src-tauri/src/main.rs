// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod input_data;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn compile_model(data: input_data::CompilationDataInput) {
    println!("Compiling model {}", data.model_name);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![compile_model])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
