#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Run the backend inside the app for as long as it is open. Its
            // database, images, backups and .env live in the app's local data
            // folder (on Windows: %LOCALAPPDATA%\com.exhibit-manager.app).
            let data_dir = app.path().app_local_data_dir()?;
            std::thread::spawn(move || {
                if let Err(e) = exhibit_manager_backend::run(data_dir) {
                    eprintln!("Backend server stopped: {e}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
