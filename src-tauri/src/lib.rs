mod models;
mod state;
mod commands;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Get the app data directory
            let data_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;
            
            // Initialize app state
            let app_state = AppState::new(data_dir)
                .map_err(|e| format!("Failed to initialize app state: {}", e))?;
            
            // Manage the app state
            app.manage(app_state);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_workspace,
            commands::get_project,
            commands::create_project,
            commands::delete_project,
            commands::switch_project,
            commands::rename_project,
            commands::create_section,
            commands::update_section_name,
            commands::delete_section,
            commands::create_topic,
            commands::update_topic_content,
            commands::update_topic_name,
            commands::delete_topic,
            commands::reorder_item,
            commands::get_merged_output,
            commands::refine_with_llm,
            commands::refine_with_llm_stream,
            commands::save_topic_refinement,
            commands::save_section_refinement,
            commands::save_project_refinement,
            commands::save_project_er_diagram,
            commands::refine_er_diagram_with_llm_stream,
            commands::save_project_uml_diagram,
            commands::refine_uml_diagram_with_llm_stream,
            commands::save_project_flowchart,
            commands::refine_flowchart_with_llm_stream,
            commands::save_project_user_journey,
            commands::save_project_user_stories,
            commands::refine_user_journey_with_llm_stream,
            commands::refine_user_stories_with_llm_stream,
            commands::get_llm_settings,
            commands::save_llm_settings,
            commands::get_platform,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
