// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod import;
pub mod input;
pub mod process;
pub mod utilities;

use import::load_all_source_files;
use input::CompilationDataInput;
use process::process;
use utilities::logging::{log, LogLevel};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn compile_model(data: CompilationDataInput) {
    log(format!("Compiling model {}.mdl!", data.model_name), LogLevel::Info);

    let loaded_source_files = match load_all_source_files(&data) {
        Ok(source_files) => source_files,
        Err(error) => {
            log(format!("Fail to compile due to: {}!", error.to_string()), LogLevel::Error);
            return;
        }
    };

    let processed_data = match process(data, loaded_source_files) {
        Ok(data) => data,
        Err(error) => {
            log(format!("Fail to compile due to: {}!", error.to_string()), LogLevel::Error);
            return;
        }
    };
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![compile_model])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
