#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apphistory;
mod collector;
mod commands;
mod error;
mod icon;
mod models;
mod pdh;
mod performance;
mod processes;
mod services;
mod startup;
mod state;
mod users;
mod util;

use std::sync::Arc;
use tauri::{Manager, WindowEvent};
use window_vibrancy::apply_mica;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cores = util::logical_processor_count();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::new(cores))
        .setup(|app| {
            let handle = app.handle().clone();
            let state: Arc<state::AppState> = app.state::<Arc<state::AppState>>().inner().clone();
            collector::spawn(handle, state);

            if let Some(window) = app.get_webview_window("main") {
                let _ = apply_mica(&window, None);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.app_handle().try_state::<Arc<state::AppState>>() {
                    state.history.lock().save();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::set_active_tab,
            commands::set_refresh_speed,
            commands::get_processes,
            commands::get_performance,
            commands::end_task,
            commands::run_new_task,
            commands::open_file_location,
            commands::open_properties,
            commands::search_online,
            commands::toggle_always_on_top,
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_close,
            commands::set_priority,
            commands::set_affinity,
            commands::efficiency_mode,
            commands::get_services,
            commands::control_service,
            commands::refresh_services,
            commands::get_startup,
            commands::set_startup_enabled,
            commands::get_users,
            commands::get_app_history,
            commands::reset_app_history,
            commands::restart_as_admin,
        ])
        .run(tauri::generate_context!())
        .expect("启动任务管理器失败");
}
