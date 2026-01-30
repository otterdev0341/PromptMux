use crate::models::{Workspace, Project, Section, Topic};
use crate::state::AppState;
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// Get the current platform/OS
#[tauri::command]
pub fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

/// Get the workspace with all projects
#[tauri::command]
pub fn get_workspace(state: State<AppState>) -> Result<Workspace, String> {
    let workspace = state.workspace.lock().unwrap();
    Ok(workspace.clone())
}

/// Get the active project
#[tauri::command]
pub fn get_project(state: State<AppState>) -> Result<Project, String> {
    let workspace = state.workspace.lock().unwrap();
    workspace
        .get_active_project()
        .ok_or("No active project found".to_string())
        .map(|p| p.clone())
}

/// Create a new project
#[tauri::command]
pub fn create_project(state: State<AppState>, name: String) -> Result<Project, String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = Project::new(name);
    let project_clone = project.clone();
    workspace.add_project(project);
    
    // Save to file
    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }
    
    Ok(project_clone)
}

/// Delete a project
#[tauri::command]
pub fn delete_project(state: State<AppState>, project_id: String) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    workspace.remove_project(&project_id)?;
    
    // Save to file
    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }
    
    Ok(())
}

/// Switch to a different project
#[tauri::command]
pub fn switch_project(state: State<AppState>, project_id: String) -> Result<Project, String> {
    let mut workspace = state.workspace.lock().unwrap();
    workspace.set_active_project(&project_id)?;
    
    let project = workspace.get_active_project()
        .ok_or("Failed to get active project".to_string())?
        .clone();
    
    // Save to file
    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }
    
    Ok(project)
}

/// Rename a project
#[tauri::command]
pub fn rename_project(
    state: State<AppState>,
    project_id: String,
    name: String,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    
    if let Some(project) = workspace.get_project_mut(&project_id) {
        project.name = name;
        project.updated_at = chrono::Utc::now().to_rfc3339();
        
        if let Err(e) = save_workspace(&workspace, &state.data_dir) {
            return Err(format!("Failed to save workspace: {}", e));
        }
        
        Ok(())
    } else {
        Err(format!("Project with id {} not found", project_id))
    }
}

#[tauri::command]
pub fn create_section(state: State<AppState>, name: String) -> Result<Section, String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;
    
    let section = Section::new(name);
    let section_clone = section.clone();
    project.add_section(section);
    
    // Save to file
    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }
    
    Ok(section_clone)
}

#[tauri::command]
pub fn update_section_name(
    state: State<AppState>,
    section_id: String,
    name: String,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;
    
    if let Some(section) = project.get_section_mut(&section_id) {
        section.name = name;
        project.updated_at = chrono::Utc::now().to_rfc3339();
        
        if let Err(e) = save_workspace(&workspace, &state.data_dir) {
            return Err(format!("Failed to save workspace: {}", e));
        }
        
        Ok(())
    } else {
        Err(format!("Section with id {} not found", section_id))
    }
}

#[tauri::command]
pub fn delete_section(state: State<AppState>, section_id: String) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;
    
    project.remove_section(&section_id)?;
    
    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }
    
    Ok(())
}

#[tauri::command]
pub fn create_topic(
    state: State<AppState>,
    section_id: String,
    name: String,
) -> Result<Topic, String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;
    
    if let Some(section) = project.get_section_mut(&section_id) {
        let topic = Topic::new(name, String::new(), section_id.clone());
        let topic_clone = topic.clone();
        section.add_topic(topic);
        project.updated_at = chrono::Utc::now().to_rfc3339();
        
        if let Err(e) = save_workspace(&workspace, &state.data_dir) {
            return Err(format!("Failed to save workspace: {}", e));
        }
        
        Ok(topic_clone)
    } else {
        Err(format!("Section with id {} not found", section_id))
    }
}

#[tauri::command]
pub fn update_topic_content(
    state: State<AppState>,
    topic_id: String,
    content: String,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;
    
    if let Some(topic) = project.get_topic_mut(&topic_id) {
        topic.content = content;
        project.updated_at = chrono::Utc::now().to_rfc3339();
        
        if let Err(e) = save_workspace(&workspace, &state.data_dir) {
            return Err(format!("Failed to save workspace: {}", e));
        }
        
        Ok(())
    } else {
        Err(format!("Topic with id {} not found", topic_id))
    }
}

#[tauri::command]
pub fn update_topic_name(
    state: State<AppState>,
    topic_id: String,
    name: String,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;
    
    if let Some(topic) = project.get_topic_mut(&topic_id) {
        topic.name = name;
        project.updated_at = chrono::Utc::now().to_rfc3339();
        
        if let Err(e) = save_workspace(&workspace, &state.data_dir) {
            return Err(format!("Failed to save workspace: {}", e));
        }
        
        Ok(())
    } else {
        Err(format!("Topic with id {} not found", topic_id))
    }
}

#[tauri::command]
pub fn delete_topic(state: State<AppState>, topic_id: String) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;
    
    // Find and remove the topic from its section
    for section in &mut project.sections {
        if section.remove_topic(&topic_id).is_ok() {
            project.updated_at = chrono::Utc::now().to_rfc3339();
            
            if let Err(e) = save_workspace(&workspace, &state.data_dir) {
                return Err(format!("Failed to save workspace: {}", e));
            }
            
            return Ok(());
        }
    }
    
    Err(format!("Topic with id {} not found", topic_id))
}

#[tauri::command]
pub fn reorder_item(
    state: State<AppState>,
    item_type: String,
    id: String,
    new_index: usize,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;
    
    project.reorder_item(&item_type, &id, new_index)?;
    
    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_merged_output(state: State<AppState>) -> Result<String, String> {
    let workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project()
        .ok_or("No active project found".to_string())?;
    Ok(project.get_merged_output())
}

#[tauri::command]
pub async fn refine_with_llm(content: String) -> Result<String, String> {
    let settings = load_settings()?;
    
    let client = reqwest::Client::new();
    
    let request_body = if settings.provider == "openai" {
        serde_json::json!({
            "model": settings.model.unwrap_or("gpt-4".to_string()),
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert at refining and improving prompts for software development projects. Your task is to take the user's prompt and make it clearer, more specific, and more effective while maintaining the original intent."
                },
                {
                    "role": "user",
                    "content": format!("Refine and improve the following prompt for a software development project:\n\n{}", content)
                }
            ]
        })
    } else if settings.provider == "anthropic" {
        serde_json::json!({
            "model": settings.model.unwrap_or("claude-3-sonnet-20240229".to_string()),
            "max_tokens": 4096,
            "messages": [
                {
                    "role": "user",
                    "content": format!("You are an expert at refining and improving prompts for software development projects. Your task is to take the user's prompt and make it clearer, more specific, and more effective while maintaining the original intent.\n\nRefine and improve the following prompt for a software development project:\n\n{}", content)
                }
            ]
        })
    } else {
        return Err(format!("Unsupported LLM provider: {}", settings.provider));
    };
    
    let url = if settings.provider == "openai" {
        format!("{}/chat/completions", settings.base_url)
    } else if settings.provider == "anthropic" {
        format!("{}/messages", settings.base_url)
    } else {
        return Err(format!("Unsupported LLM provider: {}", settings.provider));
    };
    
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to LLM API: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("LLM API error: {}", error_text));
    }
    
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;
    
    let refined_content = if settings.provider == "openai" {
        response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("Invalid OpenAI response format")?
            .to_string()
    } else if settings.provider == "anthropic" {
        response_json["content"][0]["text"]
            .as_str()
            .ok_or("Invalid Anthropic response format")?
            .to_string()
    } else {
        return Err(format!("Unsupported LLM provider: {}", settings.provider));
    };
    
    Ok(refined_content)
}

fn save_workspace(workspace: &Workspace, data_dir: &PathBuf) -> Result<(), String> {
    let workspace_path = data_dir.join("workspace.json");
    
    let workspace_json = serde_json::to_string_pretty(workspace)
        .map_err(|e| format!("Failed to serialize workspace: {}", e))?;
    
    fs::write(workspace_path, workspace_json)
        .map_err(|e| format!("Failed to write workspace file: {}", e))?;
    
    Ok(())
}

#[derive(Debug, Clone)]
struct LlmSettings {
    provider: String,
    api_key: String,
    base_url: String,
    model: Option<String>,
}

fn load_settings() -> Result<LlmSettings, String> {
    let settings_path = dirs::home_dir()
        .ok_or("Failed to get home directory")?
        .join(".promptmux")
        .join("settings.json");
    
    if !settings_path.exists() {
        return Err(
            "Settings file not found. Please create ~/.promptmux/settings.json with your LLM configuration.".to_string()
        );
    }
    
    let settings_content = fs::read_to_string(settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    
    let settings_json: serde_json::Value = serde_json::from_str(&settings_content)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;
    
    Ok(LlmSettings {
        provider: settings_json["provider"]
            .as_str()
            .unwrap_or("openai")
            .to_string(),
        api_key: settings_json["apiKey"]
            .as_str()
            .ok_or("apiKey not found in settings")?
            .to_string(),
        base_url: settings_json["baseUrl"]
            .as_str()
            .ok_or("baseUrl not found in settings")?
            .to_string(),
        model: settings_json["model"].as_str().map(|s| s.to_string()),
    })
}
