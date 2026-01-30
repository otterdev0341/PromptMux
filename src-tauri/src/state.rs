use crate::models::{Workspace, Project};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub workspace: Mutex<Workspace>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        // Ensure data directory exists
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
        
        // Try to load existing workspace, or create new one
        let workspace_path = data_dir.join("workspace.json");
        let workspace = if workspace_path.exists() {
            let workspace_json = fs::read_to_string(&workspace_path)
                .map_err(|e| format!("Failed to read workspace file: {}", e))?;
            
            serde_json::from_str(&workspace_json)
                .map_err(|e| format!("Failed to parse workspace file: {}", e))?
        } else {
            // Migrate from old single project format if it exists
            let old_project_path = data_dir.join("project.json");
            let workspace = if old_project_path.exists() {
                // Try to load old project and migrate it
                let project_json = fs::read_to_string(&old_project_path)
                    .map_err(|e| format!("Failed to read old project file: {}", e))?;
                
                let project: Project = serde_json::from_str(&project_json)
                    .map_err(|e| format!("Failed to parse old project file: {}", e))?;
                
                let active_id = project.id.clone();
                Workspace {
                    projects: vec![project],
                    active_project_id: active_id,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                }
            } else {
                // Create new workspace with default project
                Workspace::new()
            };
            
            // Save the new workspace
            let workspace_json = serde_json::to_string_pretty(&workspace)
                .map_err(|e| format!("Failed to serialize workspace: {}", e))?;
            
            fs::write(&workspace_path, workspace_json)
                .map_err(|e| format!("Failed to write workspace file: {}", e))?;
            
            workspace
        };
        
        Ok(AppState {
            workspace: Mutex::new(workspace),
            data_dir,
        })
    }
}
