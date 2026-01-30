<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  export let onClose: () => void;

  interface LlmSettings {
    provider: string;
    apiKey: string;
    baseUrl: string;
    model?: string;
    protocol?: string;
  }

  const providers = [
    { value: 'openai', label: 'OpenAI (GPT-4)', defaultBaseUrl: 'https://api.openai.com/v1', defaultModel: 'gpt-4' },
    { value: 'anthropic', label: 'Anthropic (Claude)', defaultBaseUrl: 'https://api.anthropic.com/v1', defaultModel: 'claude-3-sonnet-20240229' },
    { value: 'gemini', label: 'Google Gemini', defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai', defaultModel: 'gemini-1.5-pro' },
    { value: 'glm', label: 'Zhipu AI (GLM)', defaultBaseUrl: 'https://open.bigmodel.cn/api/paas/v4', defaultModel: 'glm-4.5-air' }
  ];

  let settings: LlmSettings = {
    provider: 'openai',
    apiKey: '',
    baseUrl: '',
    model: ''
  };

  let loading = true;
  let saving = false;
  let error = '';

  onMount(async () => {
    try {
      settings = await invoke<LlmSettings>('get_llm_settings');
      if (!settings.baseUrl) {
        updateDefaults(settings.provider);
      }
    } catch (err) {
      console.error('Failed to load settings:', err);
      // Initialize with defaults if load fails (e.g. file doesn't exist yet)
      updateDefaults('openai');
    } finally {
      loading = false;
    }
  });

  function updateDefaults(providerValue: string) {
    const provider = providers.find(p => p.value === providerValue);
    if (provider) {
      // Logic to determine correct default base URL based on protocol
      let defaultBaseUrl = provider.defaultBaseUrl;
      const currentProtocol = settings.protocol || 'openai';
      
      if (providerValue === 'glm' && currentProtocol === 'anthropic') {
         defaultBaseUrl = 'https://api.z.ai/api/anthropic';
      }

      // Only update if currently empty or set to a known default
      const isCurrentDefault = providers.some(p => p.defaultBaseUrl === settings.baseUrl) || 
                               settings.baseUrl === 'https://api.z.ai/api/anthropic' ||
                               settings.baseUrl === 'https://open.bigmodel.cn/api/paas/v4';

      if (!settings.baseUrl || isCurrentDefault) {
         settings.baseUrl = defaultBaseUrl;
      }
      
      if (!settings.model) {
        settings.model = provider.defaultModel;
      }
    }
  }

  function handleProviderChange() {
    // Reset protocol to default (OpenAI) when switching providers, unless it's explicitly Anthropic
    if (settings.provider === 'anthropic') {
        settings.protocol = 'anthropic';
    } else {
        settings.protocol = 'openai';
    }
    updateDefaults(settings.provider);
  }

  function handleProtocolChange() {
    updateDefaults(settings.provider);
  }

  async function handleSave() {
    saving = true;
    error = '';
    try {
      await invoke('save_llm_settings', { settings });
      onClose();
    } catch (err) {
      console.error('Failed to save settings:', err);
      error = 'Failed to save settings: ' + String(err);
    } finally {
      saving = false;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

<div class="modal-backdrop" on:click={handleBackdropClick} role="presentation">
  <div class="modal-content" role="dialog">
    <div class="modal-header">
      <h2>LLM Settings</h2>
      <button class="close-button" on:click={onClose}>✕</button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="loading">Loading settings...</div>
      {:else}
        {#if error}
          <div class="error-message">{error}</div>
        {/if}

        <div class="form-group">
          <label for="provider">Provider</label>
          <select id="provider" bind:value={settings.provider} on:change={handleProviderChange}>
            {#each providers as provider}
              <option value={provider.value}>{provider.label}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label for="apiKey">API Key</label>
          <input 
            type="password" 
            id="apiKey" 
            bind:value={settings.apiKey} 
            placeholder="sk-..."
          />
          <small class="hint">Your API key is stored locally in ~/.promptmux/settings.json</small>
        </div>

        <div class="form-group">
          <label for="baseUrl">Base URL</label>
          <input 
            type="text" 
            id="baseUrl" 
            bind:value={settings.baseUrl} 
            placeholder="https://api.openai.com/v1"
          />
        </div>

        <div class="form-group row">
          <label for="protocol">Protocol</label>
          <select id="protocol" bind:value={settings.protocol} on:change={handleProtocolChange}>
            <option value="openai">OpenAI Compatible (Default)</option>
            <option value="anthropic">Anthropic Compatible (Goose)</option>
          </select>
          <small class="hint">Use "Anthropic Compatible" for Goose CLI or specific proxies.</small>
        </div>

        <div class="form-group">
          <label for="model">Model Name</label>
          <input 
            type="text" 
            id="model" 
            bind:value={settings.model} 
            placeholder="gpt-4"
          />
          {#if settings.provider === 'gemini'}
            <small class="hint">Recommended: <code>gemini-1.5-pro</code>, <code>gemini-1.5-flash</code></small>
          {:else if settings.provider === 'glm'}
            <small class="hint">Recommended: <code>glm-4.7</code>, <code>glm-4.5-air</code> (for Lite)</small>
          {:else if settings.provider === 'anthropic'}
            <small class="hint">Recommended: <code>claude-3-5-sonnet-20240620</code>, <code>claude-3-opus-20240229</code></small>
          {/if}
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <button class="btn btn-secondary" on:click={onClose}>Cancel</button>
      <button class="btn btn-primary" on:click={handleSave} disabled={saving || loading}>
        {saving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  </div>
</div>

<style>
  :global(.modal-content *) {
    color-scheme: dark;
  }

  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background-color: #1a202c;
    border-radius: 0.5rem;
    max-width: 500px;
    width: 90%;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    color-scheme: dark;
  }

  .modal-header {
    padding: 1.25rem;
    border-bottom: 1px solid #2d3748;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: #e2e8f0;
  }

  .close-button {
    background: none;
    border: none;
    color: #718096;
    font-size: 1.25rem;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .modal-body {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #cbd5e0;
  }

  input, select {
    padding: 0.625rem;
    background-color: #0d1117;
    border: 1px solid #4a5568;
    border-radius: 0.375rem;
    color: #e2e8f0;
    font-size: 0.875rem;
    font-family: inherit;
    width: 100%;
    box-sizing: border-box;
    color-scheme: dark;
  }

  input:focus, select:focus {
    outline: none;
    border-color: #3182ce;
  }

  /* Fix for select dropdown options in some browsers */
  option {
    background-color: #0d1117;
    color: #e2e8f0;
  }

  .hint {
    font-size: 0.75rem;
    color: #718096;
  }

  .error-message {
    color: #f56565;
    background-color: rgba(245, 101, 101, 0.1);
    padding: 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  .modal-footer {
    padding: 1.25rem;
    border-top: 1px solid #2d3748;
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: background-color 0.15s ease;
  }

  .btn-secondary {
    background-color: #4a5568;
    color: #e2e8f0;
  }

  .btn-secondary:hover {
    background-color: #718096;
  }

  .btn-primary {
    background-color: #3182ce;
    color: white;
  }

  .btn-primary:hover {
    background-color: #2c5282;
  }
  
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
