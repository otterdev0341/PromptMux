use crate::models::{Workspace, Project, Section, Topic, Refinement};
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
pub fn save_topic_refinement(
    state: State<AppState>,
    topic_id: String,
    refinement: Refinement,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;

    if let Some(topic) = project.get_topic_mut(&topic_id) {
        topic.add_refinement(refinement);
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
pub fn save_section_refinement(
    state: State<AppState>,
    section_id: String,
    refinement: Refinement,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;

    if let Some(section) = project.get_section_mut(&section_id) {
        section.add_refinement(refinement);
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
pub fn save_project_refinement(
    state: State<AppState>,
    refinement: Refinement,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;

    project.add_refinement(refinement);
    project.updated_at = chrono::Utc::now().to_rfc3339();

    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }

    Ok(())
}

#[tauri::command]
pub fn save_project_er_diagram(
    state: State<AppState>,
    er_diagram: String,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;

    project.er_diagram = Some(er_diagram);
    project.updated_at = chrono::Utc::now().to_rfc3339();

    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }

    Ok(())
}

use tauri::Emitter;
use futures_util::StreamExt;

#[tauri::command]
pub async fn refine_with_llm_stream(app: tauri::AppHandle, content: String) -> Result<(), String> {
    let settings = load_settings()?;
    let client = reqwest::Client::new();
    
    // Determine protocol
    let is_anthropic = settings.protocol.as_deref().unwrap_or("openai") == "anthropic" 
        || (settings.protocol.is_none() && settings.provider == "anthropic");

    let request_body = if !is_anthropic {
        // OpenAI Format (Works for OpenAI, GLM, Gemini, etc.)
        serde_json::json!({
            "model": settings.model.unwrap_or("gpt-4".to_string()),
            "stream": true,
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
    } else {
        // Anthropic Format
        serde_json::json!({
            "model": settings.model.unwrap_or("claude-3-sonnet-20240229".to_string()),
            "max_tokens": 4096,
            "stream": true,
            "messages": [
                {
                    "role": "user",
                    "content": format!("You are an expert at refining and improving prompts for software development projects. Your task is to take the user's prompt and make it clearer, more specific, and more effective while maintaining the original intent.\n\nRefine and improve the following prompt for a software development project:\n\n{}", content)
                }
            ]
        })
    };
    
    let url = if !is_anthropic {
        format!("{}/chat/completions", settings.base_url.trim_end_matches('/'))
    } else {
        format!("{}/messages", settings.base_url.trim_end_matches('/'))
    };
    
    // Construct request
    let mut request_builder = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body);
        
    if is_anthropic {
         request_builder = request_builder
            .header("x-api-key", &settings.api_key)
            .header("anthropic-version", "2023-06-01");
    } else {
         request_builder = request_builder.header("Authorization", format!("Bearer {}", settings.api_key));
    }

    // Send request
    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("LLM API error: {}", error_text));
    }

    // Spawn a task to handle streaming so we don't block
    tauri::async_runtime::spawn(async move {
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let chunk_str = String::from_utf8_lossy(&bytes);
                    for line in chunk_str.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        
                        // Parse SSE
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                let _ = app.emit("refine:done", ());
                                return;
                            }
                            
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if !is_anthropic {
                                    // OpenAI format
                                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                        let _ = app.emit("refine:chunk", content);
                                    }
                                } else {
                                    // Anthropic format (data: { type: "content_block_delta", delta: { type: "text_delta", text: "..." } })
                                    if json["type"] == "content_block_delta" {
                                        if let Some(text) = json["delta"]["text"].as_str() {
                                            let _ = app.emit("refine:chunk", text);
                                        }
                                    }
                                    // Check for message_stop
                                    if json["type"] == "message_stop" {
                                        let _ = app.emit("refine:done", ());
                                        return;
                                    }
                                }
                            }
                        } 
                        // Anthropic sometimes sends event: ... lines, we mostly care about data:
                    }
                }
                Err(e) => {
                    let _ = app.emit("refine:error", format!("Stream error: {}", e));
                    return;
                }
            }
        }
        let _ = app.emit("refine:done", ());
    });

    Ok(())
}

#[tauri::command]
pub async fn refine_er_diagram_with_llm_stream(app: tauri::AppHandle, content: String) -> Result<(), String> {
    let settings = load_settings()?;
    let client = reqwest::Client::new();
    
    // Determine protocol
    let is_anthropic = settings.protocol.as_deref().unwrap_or("openai") == "anthropic" 
        || (settings.protocol.is_none() && settings.provider == "anthropic");

    let system_prompt = "You are an expert software architect. Your task is to analyze the provided software project description and generate a Mermaid ER Diagram representing the data model.
    
Output ONLY the mermaid code block. Do not include markdown code fences (```mermaid). Do not include any explanation. Just the code.
Start with `erDiagram`.
    
Example output:
erDiagram
    USER ||--o{ ORDER : places
    USER {
        string username
        string email
    }
    ORDER ||--|{ ORDER_ITEM : contains
    ORDER {
        int id
        string deliveryAddress
    }";

    let request_body = if !is_anthropic {
        // OpenAI Format
        serde_json::json!({
            "model": settings.model.unwrap_or("gpt-4".to_string()),
            "stream": true,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": format!("Generate a Mermaid ER Diagram for the following project description:\n\n{}", content)
                }
            ]
        })
    } else {
        // Anthropic Format
        serde_json::json!({
            "model": settings.model.unwrap_or("claude-3-sonnet-20240229".to_string()),
            "max_tokens": 4096,
            "stream": true,
            "messages": [
                {
                    "role": "user",
                    "content": format!("{}\n\nGenerate a Mermaid ER Diagram for the following project description:\n\n{}", system_prompt, content)
                }
            ]
        })
    };
    
    let url = if !is_anthropic {
        format!("{}/chat/completions", settings.base_url.trim_end_matches('/'))
    } else {
        format!("{}/messages", settings.base_url.trim_end_matches('/'))
    };
    
    // Construct request
    let mut request_builder = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body);
        
    if is_anthropic {
         request_builder = request_builder
            .header("x-api-key", &settings.api_key)
            .header("anthropic-version", "2023-06-01");
    } else {
         request_builder = request_builder.header("Authorization", format!("Bearer {}", settings.api_key));
    }

    // Send request
    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("LLM API error: {}", error_text));
    }

    // Spawn a task to handle streaming so we don't block
    tauri::async_runtime::spawn(async move {
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let chunk_str = String::from_utf8_lossy(&bytes);
                    for line in chunk_str.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        
                        // Parse SSE
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                let _ = app.emit("er:done", ());
                                return;
                            }
                            
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if !is_anthropic {
                                    // OpenAI format
                                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                        let _ = app.emit("er:chunk", content);
                                    }
                                } else {
                                    // Anthropic format
                                    if json["type"] == "content_block_delta" {
                                        if let Some(text) = json["delta"]["text"].as_str() {
                                            let _ = app.emit("er:chunk", text);
                                        }
                                    }
                                    if json["type"] == "message_stop" {
                                        let _ = app.emit("er:done", ());
                                        return;
                                    }
                                }
                            }
                        } 
                    }
                }
                Err(e) => {
                    let _ = app.emit("er:error", format!("Stream error: {}", e));
                    return;
                }
            }
        }
        let _ = app.emit("er:done", ());
    });

    Ok(())
}

#[tauri::command]
pub fn save_project_uml_diagram(
    state: State<AppState>,
    uml_diagram: String,
) -> Result<(), String> {
    let mut workspace = state.workspace.lock().unwrap();
    let project = workspace.get_active_project_mut()
        .ok_or("No active project found".to_string())?;

    project.uml_diagram = Some(uml_diagram);
    project.updated_at = chrono::Utc::now().to_rfc3339();

    if let Err(e) = save_workspace(&workspace, &state.data_dir) {
        return Err(format!("Failed to save workspace: {}", e));
    }

    Ok(())
}

#[tauri::command]
pub async fn refine_uml_diagram_with_llm_stream(app: tauri::AppHandle, content: String) -> Result<(), String> {
    let settings = load_settings()?;
    let client = reqwest::Client::new();
    
    // Determine protocol
    let is_anthropic = settings.protocol.as_deref().unwrap_or("openai") == "anthropic" 
        || (settings.protocol.is_none() && settings.provider == "anthropic");

    let system_prompt = "You are an expert software architect. Your task is to analyze the provided software project description and generate a Mermaid Class Diagram representing the architecture using the Repository Pattern.
    
Include the following layers where appropriate:
- Interface
- Repository
- Service
- Resource/Controller
- DTO

Output ONLY the mermaid code block. Do not include markdown code fences (```mermaid). Do not include any explanation. Just the code.
Start with `classDiagram`.

Example output:
classDiagram
    class UserRepository {
        <<interface>>
        +findById(id: Long) User
        +save(user: User) User
    }
    class UserRepositoryImpl {
        +findById(id: Long) User
        +save(user: User) User
    }
    class UserService {
        -userRepository: UserRepository
        +createUser(userDto: UserDTO) UserDTO
    }
    UserRepository <|.. UserRepositoryImpl
    UserService --> UserRepository";

    let request_body = if !is_anthropic {
        // OpenAI Format
        serde_json::json!({
            "model": settings.model.unwrap_or("gpt-4".to_string()),
            "stream": true,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": format!("Generate a Mermaid Class Diagram (Repository Pattern) for the following project description:\n\n{}", content)
                }
            ]
        })
    } else {
        // Anthropic Format
        serde_json::json!({
            "model": settings.model.unwrap_or("claude-3-sonnet-20240229".to_string()),
            "max_tokens": 4096,
            "stream": true,
            "messages": [
                {
                    "role": "user",
                    "content": format!("{}\n\nGenerate a Mermaid Class Diagram (Repository Pattern) for the following project description:\n\n{}", system_prompt, content)
                }
            ]
        })
    };
    
    let url = if !is_anthropic {
        format!("{}/chat/completions", settings.base_url.trim_end_matches('/'))
    } else {
        format!("{}/messages", settings.base_url.trim_end_matches('/'))
    };
    
    // Construct request
    let mut request_builder = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body);
        
    if is_anthropic {
         request_builder = request_builder
            .header("x-api-key", &settings.api_key)
            .header("anthropic-version", "2023-06-01");
    } else {
         request_builder = request_builder.header("Authorization", format!("Bearer {}", settings.api_key));
    }

    // Send request
    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("LLM API error: {}", error_text));
    }

    // Spawn a task to handle streaming so we don't block
    tauri::async_runtime::spawn(async move {
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let chunk_str = String::from_utf8_lossy(&bytes);
                    for line in chunk_str.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        
                        // Parse SSE
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                let _ = app.emit("uml:done", ());
                                return;
                            }
                            
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if !is_anthropic {
                                    // OpenAI format
                                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                        let _ = app.emit("uml:chunk", content);
                                    }
                                } else {
                                    // Anthropic format
                                    if json["type"] == "content_block_delta" {
                                        if let Some(text) = json["delta"]["text"].as_str() {
                                            let _ = app.emit("uml:chunk", text);
                                        }
                                    }
                                    if json["type"] == "message_stop" {
                                        let _ = app.emit("uml:done", ());
                                        return;
                                    }
                                }
                            }
                        } 
                    }
                }
                Err(e) => {
                    let _ = app.emit("uml:error", format!("Stream error: {}", e));
                    return;
                }
            }
        }
        let _ = app.emit("uml:done", ());
    });

    Ok(())
}

#[tauri::command]
pub async fn refine_with_llm(_content: String) -> Result<String, String> {
    // Legacy implementation kept for fallback if needed, or redirect to stream?
    // For now, let's keep it simple and just return error saying to use stream,
    // or keep the old implementation as is. 
    // Actually, let's keep the old implementation as a fallback for now.
    let _settings = load_settings()?;
    // ... (rest of the old implementation would go here, but for brevity/cleanliness, 
    // I am effectively replacing the old function with the streaming one in the frontend usage.
    // The user might still call this, so I will keep a minimal version or error out).
    
    Err("Please use refine_with_llm_stream".to_string())
}

fn save_workspace(workspace: &Workspace, data_dir: &PathBuf) -> Result<(), String> {
    let workspace_path = data_dir.join("workspace.json");
    
    let workspace_json = serde_json::to_string_pretty(workspace)
        .map_err(|e| format!("Failed to serialize workspace: {}", e))?;
    
    fs::write(workspace_path, workspace_json)
        .map_err(|e| format!("Failed to write workspace file: {}", e))?;
    
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmSettings {
    pub provider: String,
    pub api_key: String,
    pub base_url: String,
    pub model: Option<String>,
    pub protocol: Option<String>,
}

#[tauri::command]
pub fn get_llm_settings() -> Result<LlmSettings, String> {
    load_settings()
}

#[tauri::command]
pub fn save_llm_settings(settings: LlmSettings) -> Result<(), String> {
    let home_dir = dirs::home_dir()
        .ok_or("Failed to get home directory")?;
    let settings_dir = home_dir.join(".promptmux");
    let settings_path = settings_dir.join("settings.json");
    
    if !settings_dir.exists() {
        fs::create_dir_all(&settings_dir)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }
    
    let settings_json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
    fs::write(settings_path, settings_json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
    Ok(())
}

fn load_settings() -> Result<LlmSettings, String> {
    let settings_path = dirs::home_dir()
        .ok_or("Failed to get home directory")?
        .join(".promptmux")
        .join("settings.json");
    
    if !settings_path.exists() {
        // Return default settings if not found
        return Ok(LlmSettings {
            provider: "openai".to_string(),
            api_key: "".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: Some("gpt-4".to_string()),
            protocol: Some("openai".to_string()),
        });
    }
    
    let settings_content = fs::read_to_string(settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    
    let mut settings: LlmSettings = serde_json::from_str(&settings_content)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    // Backfill protocol if missing
    if settings.protocol.is_none() {
        settings.protocol = Some(if settings.provider == "anthropic" {
            "anthropic".to_string()
        } else {
            "openai".to_string()
        });
    }
    
    Ok(settings)
}
