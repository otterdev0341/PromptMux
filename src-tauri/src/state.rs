use crate::models::Project;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub project: Mutex<Project>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        // Ensure data directory exists
        fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
        
        // Try to load existing project, or create new one
        let project_path = data_dir.join("project.json");
        let project = if project_path.exists() {
            let project_json = fs::read_to_string(&project_path)
                .map_err(|e| format!("Failed to read project file: {}", e))?;
            
            serde_json::from_str(&project_json)
                .map_err(|e| format!("Failed to parse project file: {}", e))?
        } else {
            let new_project = Project::new("My Project".to_string());
            
            // Save the new project
            let project_json = serde_json::to_string_pretty(&new_project)
                .map_err(|e| format!("Failed to serialize project: {}", e))?;
            
            fs::write(&project_path, project_json)
                .map_err(|e| format!("Failed to write project file: {}", e))?;
            
            new_project
        };
        
        Ok(AppState {
            project: Mutex::new(project),
            data_dir,
        })
    }
}
