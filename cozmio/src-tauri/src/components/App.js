/**
 * App - Main application component integrating all UI panels
 */

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

import { createStatusPanel, updateStatusPanel, initStateUpdateListener, fetchAndApplyRunningState } from './StatusPanel.js';
import { createHistoryList, loadHistory } from './HistoryList.js';
import { createSettings, loadSettings } from './Settings.js';
import { createMemoryInspector, loadMemoryInspector } from './MemoryInspector.js';
import { buildExecutionPresence, presenceStatusLabel } from './PresenceModel.js';

/**
 * Mount the application to a DOM element
 * @param {HTMLElement} el - The container element
 */
export function mount(el) {
    // Build the main app HTML structure
    el.innerHTML = `
        <!-- Title Bar -->
        <div class="title-bar">
            <div class="title-bar-drag">
                <span class="header-state-dot" id="header-state-dot"></span>
                <span class="header-state-label" id="header-state-label">空闲</span>
                <span class="app-title">Cozmio</span>
            </div>
            <div class="window-controls">
                <button class="window-btn minimize" id="minimize-btn" title="Minimize">&#x2212;</button>
                <button class="window-btn maximize" id="maximize-btn" title="Maximize">&#x25A1;</button>
                <button class="window-btn close" id="close-btn" title="Close">&#x2715;</button>
            </div>
        </div>

        <!-- Main Container -->
        <div class="main-container">
            <!-- Sidebar -->
            <nav class="sidebar" id="sidebar">
                <div class="sidebar-brand">Cozmio</div>
                <ul class="sidebar-nav">
                    <li class="nav-item active" data-panel="status">
                        <span class="nav-icon">&#x25CF;</span>
                        <span class="nav-text">STATUS</span>
                    </li>
                    <li class="nav-item" data-panel="history">
                        <span class="nav-icon">&#x23F0;</span>
                        <span class="nav-text">HISTORY</span>
                    </li>
                    <li class="nav-item" data-panel="memory">
                        <span class="nav-icon">&#x25A3;</span>
                        <span class="nav-text">MEMORY</span>
                    </li>
                    <li class="nav-item" data-panel="settings">
                        <span class="nav-icon">&#x2699;</span>
                        <span class="nav-text">CONFIG</span>
                    </li>
                </ul>
            </nav>

            <!-- Panel Container -->
            <div class="panel-container" id="panel-container">
                <!-- Panels will be dynamically inserted here -->
            </div>
        </div>
    `;

    // Initialize the app
    initApp(el);
}

/**
 * Initialize the application
 * @param {HTMLElement} el - The container element
 */
async function initApp(el) {
    const panelContainer = el.querySelector('#panel-container');

    // 1. Create all panels
    const statusPanel = createStatusPanel({});
    const historyPanel = createHistoryList();
    const memoryPanel = createMemoryInspector();
    const settingsPanel = createSettings();

    // 2. Add panels to container (status panel is first/active by default)
    panelContainer.appendChild(statusPanel);
    panelContainer.appendChild(historyPanel);
    panelContainer.appendChild(memoryPanel);
    panelContainer.appendChild(settingsPanel);
    showPanel(panelContainer, 'status');

    // 3. Bind sidebar navigation events
    bindSidebarEvents(el, panelContainer);

    // 4. Bind window control events
    bindWindowControls(el);

    // 5. Set up state update listener
    await initStateUpdateListener(statusPanel);

    // 6. Set up header dot state sync
    await initHeaderDotState();

    // 7. Load initial state
    await fetchAndApplyRunningState(statusPanel);

    // Load initial history
    await loadHistory(historyPanel);

    // Load initial memory snapshot
    await loadMemoryInspector(memoryPanel);

    // Load initial settings
    await loadSettings(settingsPanel);
}

/**
 * Bind sidebar navigation click events
 * @param {HTMLElement} el - The container element
 * @param {HTMLElement} panelContainer - The panel container element
 */
function bindSidebarEvents(el, panelContainer) {
    const navItems = el.querySelectorAll('.nav-item');

    navItems.forEach((item) => {
        item.addEventListener('click', () => {
            const panelName = item.dataset.panel;

            // Update active nav item
            navItems.forEach((nav) => nav.classList.remove('active'));
            item.classList.add('active');

            // Show corresponding panel
            showPanel(panelContainer, panelName);
        });
    });
}

/**
 * Show a specific panel by name
 * @param {HTMLElement} panelContainer - The panel container element
 * @param {string} panelName - The panel name (status, history, settings)
 */
function showPanel(panelContainer, panelName) {
    const panels = panelContainer.querySelectorAll('.content-panel');
    panels.forEach((panel) => {
        panel.classList.remove('active');
    });

    const targetPanel = panelContainer.querySelector(`#panel-${panelName}`);
    if (targetPanel) {
        targetPanel.classList.add('active');
        if (panelName === 'history') {
            loadHistory(targetPanel).catch((err) => {
                console.error('Failed to refresh history:', err);
            });
        } else if (panelName === 'memory') {
            loadMemoryInspector(targetPanel).catch((err) => {
                console.error('Failed to refresh memory inspector:', err);
            });
        }
    }
}

/**
 * Bind window control button events
 * @param {HTMLElement} el - The container element
 */
function bindWindowControls(el) {
    const minimizeBtn = el.querySelector('#minimize-btn');
    const maximizeBtn = el.querySelector('#maximize-btn');
    const closeBtn = el.querySelector('#close-btn');

    if (minimizeBtn) {
        minimizeBtn.addEventListener('click', async () => {
            try {
                const window = getCurrentWindow();
                await window.minimize();
            } catch (err) {
                console.error('Failed to minimize:', err);
            }
        });
    }

    if (maximizeBtn) {
        maximizeBtn.addEventListener('click', async () => {
            try {
                const window = getCurrentWindow();
                const isMaximized = await window.isMaximized();
                if (isMaximized) {
                    await window.unmaximize();
                } else {
                    await window.maximize();
                }
            } catch (err) {
                console.error('Failed to toggle maximize:', err);
            }
        });
    }

    if (closeBtn) {
        closeBtn.addEventListener('click', async () => {
            try {
                const window = getCurrentWindow();
                await window.hide();
            } catch (err) {
                console.error('Failed to hide window:', err);
            }
        });
    }
}

/**
 * Sync the header dot with backend state via state-update events.
 * Mirrors MiniDot's computeMiniState logic so the header dot matches
 * the floating dot's color at all times.
 */
async function initHeaderDotState() {
    const { listen } = window.__TAURI__.event;

    function applyDot(overallStatus) {
        const dot = document.getElementById('header-state-dot');
        const label = document.getElementById('header-state-label');
        if (!dot) return;
        dot.className = 'header-state-dot';
        if (overallStatus) dot.classList.add(overallStatus);
        if (label) label.textContent = presenceStatusLabel(overallStatus);
    }

    // Listen for state updates and update dot
    await listen('state-update', (event) => {
        const presence = buildExecutionPresence(event.payload || {});
        applyDot(presence.overall_status);
    });

    // Fetch initial state
    try {
        const initialState = await invoke('get_ui_state');
        const presence = buildExecutionPresence(initialState || {});
        applyDot(presence.overall_status);
    } catch (err) {
        console.error('[App] Failed to sync header dot state:', err);
    }
}
