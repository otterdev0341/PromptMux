import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Topic {
  id: string;
  name: string;
  content: string;
  order_index: number;
  section_id: string;
}

export interface Section {
  id: string;
  name: string;
  order_index: number;
  topics: Topic[];
}

export interface Project {
  id: string;
  name: string;
  sections: Section[];
  created_at: string;
  updated_at: string;
}

export const projectStore = writable<Project | null>(null);
export const activeTopicId = writable<string | null>(null);
export const activeSectionId = writable<string | null>(null);
export const platform = writable<string>('unknown');

export const mergedOutput = derived(
  projectStore,
  ($project) => {
    if (!$project) return "";
    
    const sortedSections = [...$project.sections].sort((a, b) => a.order_index - b.order_index);
    
    return sortedSections.map(section => {
      const sortedTopics = [...section.topics].sort((a, b) => a.order_index - b.order_index);
      const topicContent = sortedTopics
        .map(topic => topic.content.trim())
        .filter(content => content.length > 0)
        .join('\n\n');
      
      return `// Section: ${section.name}\n${topicContent}`;
    }).join('\n\n---\n\n');
  }
);

export const activeTopic = derived(
  [projectStore, activeTopicId],
  ([$project, $activeTopicId]) => {
    if (!$project || !$activeTopicId) return null;
    
    for (const section of $project.sections) {
      const topic = section.topics.find(t => t.id === $activeTopicId);
      if (topic) return { ...topic, sectionName: section.name };
    }
    
    return null;
  }
);

export async function loadProject(): Promise<void> {
  try {
    const project = await invoke<Project>('get_project');
    projectStore.set(project);
  } catch (error) {
    console.error('Failed to load project:', error);
    throw error;
  }
}

export async function createSection(name: string): Promise<Section> {
  try {
    const section = await invoke<Section>('create_section', { name });
    await loadProject();
    return section;
  } catch (error) {
    console.error('Failed to create section:', error);
    throw error;
  }
}

export async function updateSectionName(sectionId: string, name: string): Promise<void> {
  try {
    await invoke('update_section_name', { sectionId, name });
    await loadProject();
  } catch (error) {
    console.error('Failed to update section name:', error);
    throw error;
  }
}

export async function deleteSection(sectionId: string): Promise<void> {
  try {
    await invoke('delete_section', { sectionId });
    await loadProject();
    
    // Clear active selection if it was the deleted section
    const currentSectionId = get(activeSectionId);
    if (currentSectionId === sectionId) {
      activeSectionId.set(null);
      activeTopicId.set(null);
    }
  } catch (error) {
    console.error('Failed to delete section:', error);
    throw error;
  }
}

export async function createTopic(sectionId: string, name: string): Promise<Topic> {
  try {
    const topic = await invoke<Topic>('create_topic', { sectionId, name });
    await loadProject();
    return topic;
  } catch (error) {
    console.error('Failed to create topic:', error);
    throw error;
  }
}

export async function updateTopicContent(topicId: string, content: string): Promise<void> {
  try {
    await invoke('update_topic_content', { topicId, content });
    await loadProject();
  } catch (error) {
    console.error('Failed to update topic content:', error);
    throw error;
  }
}

export async function updateTopicName(topicId: string, name: string): Promise<void> {
  try {
    await invoke('update_topic_name', { topicId, name });
    await loadProject();
  } catch (error) {
    console.error('Failed to update topic name:', error);
    throw error;
  }
}

export async function deleteTopic(topicId: string): Promise<void> {
  try {
    await invoke('delete_topic', { topicId });
    await loadProject();
    
    // Clear active selection if it was the deleted topic
    const currentTopicId = get(activeTopicId);
    if (currentTopicId === topicId) {
      activeTopicId.set(null);
    }
  } catch (error) {
    console.error('Failed to delete topic:', error);
    throw error;
  }
}

export async function reorderItem(itemType: string, id: string, newIndex: number): Promise<void> {
  try {
    await invoke('reorder_item', { itemType, id, newIndex });
    await loadProject();
  } catch (error) {
    console.error('Failed to reorder item:', error);
    throw error;
  }
}

export async function getMergedOutput(): Promise<string> {
  try {
    return await invoke<string>('get_merged_output');
  } catch (error) {
    console.error('Failed to get merged output:', error);
    throw error;
  }
}

export async function refineWithLlm(content: string): Promise<string> {
  try {
    return await invoke<string>('refine_with_llm', { content });
  } catch (error) {
    console.error('Failed to refine with LLM:', error);
    throw error;
  }
}

export async function getPlatform(): Promise<string> {
  try {
    const os = await invoke<string>('get_platform');
    // Format the OS name for display
    switch (os) {
      case 'linux':
        return 'Linux';
      case 'windows':
        return 'Windows';
      case 'macos':
        return 'macOS';
      default:
        return os.charAt(0).toUpperCase() + os.slice(1);
    }
  } catch (error) {
    console.error('Failed to get platform:', error);
    return 'Unknown';
  }
}
