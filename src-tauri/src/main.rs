// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod import;
pub mod input;
pub mod process;
pub mod utilities;
pub mod write;

use import::FileManager;
use input::ImputedCompilationData;
use process::process;
use tauri::Manager;
use utilities::logging::{log, LogLevel, LOGGER};
use write::write_files;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn compile_model(data: ImputedCompilationData, file_manager: tauri::State<FileManager>) {
    if data.model_name.is_empty() {
        log("Model name is empty!", LogLevel::Error);
        return;
    }

    log(format!("Compiling model {}.mdl!", &data.model_name), LogLevel::Info);

    let processed_data = match process(&data, &file_manager) {
        Ok(data) => data,
        Err(error) => {
            log(format!("Fail to compile due to: {}!", error), LogLevel::Error);
            return;
        }
    };

    log(String::from("Writing Files!"), LogLevel::Info);

    match write_files(data.model_name, processed_data, data.export_path) {
        Ok(_) => {}
        Err(error) => {
            log(format!("Fail to write files due to: {}!", error), LogLevel::Error);
            return;
        }
    }

    log("Model compiled successfully!", LogLevel::Info);
}

#[tauri::command]
fn load_file(path: String, file_manager: tauri::State<FileManager>) -> bool {
    match file_manager.load_file(path) {
        Ok(_) => true,
        Err(error) => {
            log(format!("Fail to load file due to: {}!", error), LogLevel::Error);
            false
        }
    }
}

#[tauri::command]
fn unload_file(path: String, file_manager: tauri::State<FileManager>) {
    file_manager.unload_file(path);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(FileManager::default())
        .setup(|app| {
            let window = app.get_webview_window("main");
            unsafe { LOGGER = window };
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![compile_model, load_file, unload_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
