/**
 * Settings - Configuration panel for model, monitor, and execution settings
 */

const { invoke } = window.__TAURI__.core;

/**
 * Create the settings panel element
 * @returns {HTMLElement} The panel element
 */
export function createSettings() {
    const panel = document.createElement('div');
    panel.id = 'panel-settings';
    panel.className = 'content-panel';

    panel.innerHTML = `
        <div class="panel-title">CONFIG</div>

        <!-- MODEL Configuration Section -->
        <div class="config-section">
            <div class="section-title">MODEL</div>
            <div class="form-group">
                <label class="form-label" for="ollama-url">Ollama URL:</label>
                <input type="text" id="ollama-url" class="form-input" placeholder="http://localhost:11434">
            </div>
            <div class="form-group">
                <label class="form-label" for="model-name">Model Name:</label>
                <div class="input-with-suffix" style="display: flex; gap: 8px;">
                    <select id="model-name" class="form-input" style="flex: 1;">
                        <option value="">-- Select Model --</option>
                    </select>
                    <button class="btn btn-secondary" id="discover-models-btn" type="button">Refresh</button>
                </div>
                <div id="discover-status" style="font-size: 12px; color: #888; margin-top: 4px;"></div>
            </div>
        </div>

        <!-- MONITOR Configuration Section -->
        <div class="config-section">
            <div class="section-title">MONITOR</div>
            <div class="form-group">
                <label class="form-label" for="poll-interval">Poll Interval:</label>
                <div class="input-with-suffix">
                    <input type="number" id="poll-interval" class="form-input" min="1" max="60" value="3">
                    <span class="input-suffix">sec</span>
                </div>
            </div>
            <div class="form-group">
                <label class="form-checkbox">
                    <input type="checkbox" id="window-change-detection">
                    <span>Window change detection</span>
                </label>
            </div>
        </div>

        <!-- EXECUTION Configuration Section -->
        <div class="config-section">
            <div class="section-title">EXECUTION</div>
            <div class="form-group">
                <label class="form-checkbox">
                    <input type="checkbox" id="execute-auto">
                    <span>Auto-execute on CONFIDENCE &gt; 0.9</span>
                </label>
            </div>
            <div class="form-group">
                <label class="form-checkbox">
                    <input type="checkbox" id="request-native-dialog">
                    <span>Use native dialog for REQUEST level</span>
                </label>
            </div>
            <div class="form-group">
                <label class="form-label" for="execute-delay">Execute Delay:</label>
                <div class="input-with-suffix">
                    <input type="number" id="execute-delay" class="form-input" min="0" max="30" value="1">
                    <span class="input-suffix">sec</span>
                </div>
            </div>
        </div>

        <!-- Action Buttons -->
        <div class="btn-group" style="margin-top: 20px; justify-content: flex-end;">
            <button class="btn btn-secondary" id="reset-btn">RESET</button>
            <button class="btn btn-primary" id="save-btn">SAVE</button>
        </div>
    `;

    // Bind reset button
    const resetBtn = panel.querySelector('#reset-btn');
    resetBtn.addEventListener('click', () => {
        loadSettings(panel, true);
    });

    // Bind save button
    const saveBtn = panel.querySelector('#save-btn');
    saveBtn.addEventListener('click', () => {
        saveSettings(panel);
    });

    // Bind discover models button
    const discoverBtn = panel.querySelector('#discover-models-btn');
    discoverBtn.addEventListener('click', () => {
        discoverModels(panel);
    });

    return panel;
}

/**
 * Discover available models from Ollama
 */
async function discoverModels(panel) {
    const statusDiv = panel.querySelector('#discover-status');
    const select = panel.querySelector('#model-name');
    const ollamaUrl = panel.querySelector('#ollama-url')?.value || 'http://localhost:11434';

    statusDiv.textContent = 'Discovering...';
    select.innerHTML = '<option value="">-- Discovering --</option>';

    try {
        const models = await invoke('list_models');

        select.innerHTML = '<option value="">-- Select Model --</option>';
        if (models && models.length > 0) {
            models.forEach(model => {
                const option = document.createElement('option');
                option.value = model;
                option.textContent = model;
                select.appendChild(option);
            });
            statusDiv.textContent = `Found ${models.length} model(s)`;
            // Auto-select the first model
            select.value = models[0];
        } else {
            statusDiv.textContent = 'No models found';
        }
    } catch (err) {
        console.error('Failed to discover models:', err);
        select.innerHTML = '<option value="">-- Error --</option>';
        statusDiv.textContent = 'Discovery failed: ' + err;
    }
}

/**
 * Load settings into the panel
 * @param {HTMLElement} panel - The panel element
 * @param {boolean} reset - If true, load default values instead of from config
 */
export async function loadSettings(panel, reset = false) {
    try {
        let config;

        if (reset) {
            // Load default configuration
            config = {
                ollama_url: 'http://localhost:11434',
                model_name: 'llava',
                poll_interval_secs: 3,
                window_change_detection: true,
                execute_auto: true,
                request_use_native_dialog: true,
                execute_delay_secs: 1
            };
        } else {
            // Load from config file
            config = await invoke('get_config');
        }

        // Populate form fields
        const ollamaUrlInput = panel.querySelector('#ollama-url');
        const modelNameInput = panel.querySelector('#model-name');
        const pollIntervalInput = panel.querySelector('#poll-interval');
        const windowChangeCheckbox = panel.querySelector('#window-change-detection');
        const executeAutoCheckbox = panel.querySelector('#execute-auto');
        const requestNativeDialogCheckbox = panel.querySelector('#request-native-dialog');
        const executeDelayInput = panel.querySelector('#execute-delay');

        if (ollamaUrlInput) ollamaUrlInput.value = config.ollama_url || '';
        // For select elements, ensure the saved model exists as an option first
        if (modelNameInput) {
            const savedModel = config.model_name || '';
            // Check if the saved model already exists in options
            const existingOption = Array.from(modelNameInput.options).find(opt => opt.value === savedModel);
            if (!existingOption && savedModel) {
                // Add the saved model as an option if it doesn't exist
                const option = document.createElement('option');
                option.value = savedModel;
                option.textContent = savedModel + ' (saved)';
                modelNameInput.appendChild(option);
            }
            modelNameInput.value = savedModel;
        }
        if (pollIntervalInput) pollIntervalInput.value = config.poll_interval_secs || 3;
        if (windowChangeCheckbox) windowChangeCheckbox.checked = config.window_change_detection !== false;
        if (executeAutoCheckbox) executeAutoCheckbox.checked = config.execute_auto === true;
        if (requestNativeDialogCheckbox) requestNativeDialogCheckbox.checked = config.request_use_native_dialog !== false;
        if (executeDelayInput) executeDelayInput.value = config.execute_delay_secs || 1;

    } catch (err) {
        console.error('Failed to load settings:', err);
        alert('Failed to load settings: ' + err);
    }
}

/**
 * Save settings from the panel to config file
 * @param {HTMLElement} panel - The panel element
 */
async function saveSettings(panel) {
    try {
        const config = {
            ollama_url: panel.querySelector('#ollama-url')?.value || '',
            model_name: panel.querySelector('#model-name')?.value || '',
            poll_interval_secs: parseInt(panel.querySelector('#poll-interval')?.value, 10) || 3,
            window_change_detection: panel.querySelector('#window-change-detection')?.checked || false,
            execute_auto: panel.querySelector('#execute-auto')?.checked || false,
            request_use_native_dialog: panel.querySelector('#request-native-dialog')?.checked || false,
            execute_delay_secs: parseInt(panel.querySelector('#execute-delay')?.value, 10) || 1
        };

        await invoke('save_config', { config });
        alert('Settings saved successfully!');
    } catch (err) {
        console.error('Failed to save settings:', err);
        alert('Failed to save settings: ' + err);
    }
}