mod commands;
mod dto;
mod state;
use dto::ProjectInfo;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::project::create_project,
            commands::project::edit_project,
            commands::project::open_project,
            commands::project::save_project,
            commands::project::close_project,
            commands::project::get_project_info,
            commands::task::get_task,
            commands::task::get_tasks,
            commands::task::add_task,
            commands::task::edit_task,
            commands::task::delete_task,
            commands::task::get_task_tree,
            commands::task::recalculate_critical_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
