use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub projects: Vec<Project>,
    pub active_project_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Workspace {
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        // Create a default project
        let default_project = Project::new("My Project".to_string());
        let active_id = default_project.id.clone();
        
        Workspace {
            projects: vec![default_project],
            active_project_id: active_id,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn add_project(&mut self, project: Project) -> String {
        self.projects.push(project.clone());
        self.updated_at = chrono::Utc::now().to_rfc3339();
        project.id.clone()
    }

    pub fn remove_project(&mut self, project_id: &str) -> Result<(), String> {
        if self.projects.len() <= 1 {
            return Err("Cannot delete the last project".to_string());
        }
        
        let original_len = self.projects.len();
        self.projects.retain(|p| p.id != project_id);
        
        if self.projects.len() < original_len {
            // If we deleted the active project, switch to another
            if self.active_project_id == project_id {
                self.active_project_id = self.projects.first()
                    .ok_or("No projects available")?
                    .id
                    .clone();
            }
            self.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("Project with id {} not found", project_id))
        }
    }

    pub fn get_active_project(&self) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == self.active_project_id)
    }

    pub fn get_active_project_mut(&mut self) -> Option<&mut Project> {
        self.projects.iter_mut().find(|p| p.id == self.active_project_id)
    }

    pub fn get_project(&self, project_id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == project_id)
    }

    pub fn get_project_mut(&mut self, project_id: &str) -> Option<&mut Project> {
        self.projects.iter_mut().find(|p| p.id == project_id)
    }

    pub fn set_active_project(&mut self, project_id: &str) -> Result<(), String> {
        if self.projects.iter().any(|p| p.id == project_id) {
            self.active_project_id = project_id.to_string();
            self.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("Project with id {} not found", project_id))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub sections: Vec<Section>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub history: Vec<Refinement>,
    pub er_diagram: Option<String>,
    pub uml_diagram: Option<String>,
    pub flowchart: Option<String>,
    pub user_journey: Option<String>,
    pub user_stories: Option<String>,
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Project {
            id: Uuid::new_v4().to_string(),
            name,
            sections: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            history: Vec::new(),
            er_diagram: None,
            uml_diagram: None,
            flowchart: None,
            user_journey: None,
            user_stories: None,
        }
    }

    pub fn add_section(&mut self, mut section: Section) {
        section.order_index = self.sections.len();
        self.sections.push(section);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn remove_section(&mut self, section_id: &str) -> Result<(), String> {
        let original_len = self.sections.len();
        self.sections.retain(|s| s.id != section_id);
        
        if self.sections.len() < original_len {
            // Reorder remaining sections
            for (index, section) in self.sections.iter_mut().enumerate() {
                section.order_index = index;
            }
            self.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("Section with id {} not found", section_id))
        }
    }

    pub fn get_section_mut(&mut self, section_id: &str) -> Option<&mut Section> {
        self.sections.iter_mut().find(|s| s.id == section_id)
    }

    pub fn get_section(&self, section_id: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.id == section_id)
    }

    pub fn get_topic_mut(&mut self, topic_id: &str) -> Option<&mut Topic> {
        self.sections
            .iter_mut()
            .find_map(|section| section.get_topic_mut(topic_id))
    }

    pub fn get_topic(&self, topic_id: &str) -> Option<&Topic> {
        self.sections
            .iter()
            .find_map(|section| section.get_topic(topic_id))
    }

    pub fn reorder_item(&mut self, item_type: &str, id: &str, new_index: usize) -> Result<(), String> {
        match item_type {
            "section" => {
                let current_index = self.sections
                    .iter()
                    .position(|s| s.id == id)
                    .ok_or("Section not found")?;
                
                if current_index == new_index {
                    return Ok(());
                }

                let section = self.sections.remove(current_index);
                let adjusted_index = if new_index > current_index {
                    new_index - 1
                } else {
                    new_index
                };
                self.sections.insert(adjusted_index, section);
                
                for (index, section) in self.sections.iter_mut().enumerate() {
                    section.order_index = index;
                }
            }
            "topic" => {
                // Find which section contains the topic
                let section = self.sections
                    .iter_mut()
                    .find(|s| s.topics.iter().any(|t| t.id == id))
                    .ok_or("Topic not found in any section")?;

                section.reorder_topic(id, new_index)?;
            }
            _ => return Err(format!("Invalid item type: {}", item_type)),
        }

        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn get_merged_output(&self) -> String {
        let mut sorted_sections = self.sections.clone();
        sorted_sections.sort_by_key(|s| s.order_index);

        sorted_sections
            .iter()
            .map(|section| {
                let mut sorted_topics = section.topics.clone();
                sorted_topics.sort_by_key(|t| t.order_index);

                let topic_content = sorted_topics
                    .iter()
                    .map(|topic| topic.content.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n\n");

                format!("// Section: {}\n{}", section.name, topic_content)
            })
            .collect::<Vec<String>>()
            .join("\n\n---\n\n")
    }

    pub fn add_refinement(&mut self, refinement: Refinement) {
        self.history.push(refinement);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub name: String,
    pub order_index: usize,
    pub topics: Vec<Topic>,
    #[serde(default)]
    pub history: Vec<Refinement>,
}

impl Section {
    pub fn new(name: String) -> Self {
        Section {
            id: Uuid::new_v4().to_string(),
            name,
            order_index: 0,
            topics: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn add_topic(&mut self, mut topic: Topic) {
        topic.order_index = self.topics.len();
        topic.section_id = self.id.clone();
        self.topics.push(topic);
    }

    pub fn remove_topic(&mut self, topic_id: &str) -> Result<(), String> {
        let original_len = self.topics.len();
        self.topics.retain(|t| t.id != topic_id);
        
        if self.topics.len() < original_len {
            // Reorder remaining topics
            for (index, topic) in self.topics.iter_mut().enumerate() {
                topic.order_index = index;
            }
            Ok(())
        } else {
            Err(format!("Topic with id {} not found", topic_id))
        }
    }

    pub fn get_topic_mut(&mut self, topic_id: &str) -> Option<&mut Topic> {
        self.topics.iter_mut().find(|t| t.id == topic_id)
    }

    pub fn get_topic(&self, topic_id: &str) -> Option<&Topic> {
        self.topics.iter().find(|t| t.id == topic_id)
    }

    pub fn reorder_topic(&mut self, topic_id: &str, new_index: usize) -> Result<(), String> {
        let current_index = self.topics
            .iter()
            .position(|t| t.id == topic_id)
            .ok_or("Topic not found")?;
        
        if current_index == new_index {
            return Ok(());
        }

        let topic = self.topics.remove(current_index);
        let adjusted_index = if new_index > current_index {
            new_index - 1
        } else {
            new_index
        };
        self.topics.insert(adjusted_index, topic);
        
        for (index, topic) in self.topics.iter_mut().enumerate() {
            topic.order_index = index;
        }

        Ok(())
    }

    pub fn add_refinement(&mut self, refinement: Refinement) {
        self.history.push(refinement);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refinement {
    pub id: String,
    pub original_content: String,
    pub refined_content: String,
    pub timestamp: String,
    #[serde(default)]
    pub kind: Option<String>, // 'text', 'er', 'uml', 'flowchart', 'journey'
    #[serde(default)]
    pub mode: Option<String>, // 'edit', 'ask'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub content: String,
    pub order_index: usize,
    pub section_id: String,
    #[serde(default)] // For backward compatibility
    pub history: Vec<Refinement>,
}

impl Topic {
    pub fn new(name: String, content: String, section_id: String) -> Self {
        Topic {
            id: Uuid::new_v4().to_string(),
            name,
            content,
            order_index: 0,
            section_id,
            history: Vec::new(),
        }
    }

    pub fn add_refinement(&mut self, refinement: Refinement) {
        self.history.push(refinement);
    }
}
