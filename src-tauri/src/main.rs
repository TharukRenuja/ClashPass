#![windows_subsystem = "windows"]

mod commands;
mod export;
mod models;
mod parser;

use std::sync::Mutex;

fn main() {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(commands::AppState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::import_files,
            commands::remove_file,
            commands::get_files,
            commands::get_groups,
            commands::resolve_group,
            commands::clear_resolve,
            commands::edit_field,
            commands::export_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
