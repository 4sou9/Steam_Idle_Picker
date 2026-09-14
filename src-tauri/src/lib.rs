mod commands;
mod idle_manager;
mod models;
mod process_guard;
mod services;
mod steam_client;

use idle_manager::IdleManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .setup(|app| {
            let resource_dir = app
                .path()
                .resource_dir()
                .unwrap_or_else(|_| std::env::current_exe().unwrap().parent().unwrap().to_path_buf());
            services::storage::migrate_legacy_data();
            let idle_manager = IdleManager::new(resource_dir);
            idle_manager.kill_orphans();
            app.manage(idle_manager);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let idle_manager = window.state::<IdleManager>();
                idle_manager.stop_all();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_cache,
            commands::refresh_library,
            commands::start_idle,
            commands::stop_idle,
            commands::stop_all,
            commands::is_idling,
            commands::get_idling_ids,
            commands::take_idle_failures,
            commands::load_settings,
            commands::save_settings,
            commands::open_steam_store,
            commands::open_steam_library,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                app_handle.state::<IdleManager>().stop_all();
            }
        });
}
