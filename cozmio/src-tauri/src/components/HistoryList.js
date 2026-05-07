/**
 * HistoryList - Displays action history list with detail view
 */

const { invoke } = window.__TAURI__.core;

/**
 * Create the history list panel
 * @returns {HTMLElement} The panel element
 */
export function createHistoryList() {
    const panel = document.createElement('div');
    panel.id = 'panel-history';
    panel.className = 'content-panel';

    panel.innerHTML = `
        <div class="panel-title">HISTORY</div>

        <!-- History List -->
        <div class="history-list" id="history-list">
            <div class="history-header">
                <span class="col-time">time</span>
                <span class="col-level">level</span>
                <span class="col-title">window</span>
            </div>
            <div id="history-items"></div>
        </div>

        <div class="divider"></div>

        <!-- Selected Detail Card -->
        <div class="module-card" id="history-detail">
            <div class="module-header">
                <span class="module-name">SELECTED RECORD</span>
                <button class="btn btn-secondary" id="clear-history-btn">CLEAR</button>
            </div>
            <div id="detail-content">
                <div class="empty-state">Select a record to view details</div>
            </div>
        </div>
    `;

    // Bind clear history button
    const clearBtn = panel.querySelector('#clear-history-btn');
    clearBtn.addEventListener('click', async () => {
        try {
            await invoke('clear_history');
            // Reload history after clearing
            const historyItems = panel.querySelector('#history-items');
            if (historyItems) {
                historyItems.innerHTML = '<div class="empty-state">No history available</div>';
            }
            const detailContent = panel.querySelector('#detail-content');
            if (detailContent) {
                detailContent.innerHTML = '<div class="empty-state">Select a record to view details</div>';
            }
        } catch (err) {
            console.error('Failed to clear history:', err);
        }
    });

    return panel;
}

/**
 * Load history records into the panel
 * @param {HTMLElement} panel - The panel element
 */
export async function loadHistory(panel) {
    try {
        // Try new session-based API first
        const sessions = await invoke('get_execution_sessions', { limit: 20 });
        if (sessions && sessions.length > 0) {
            renderSessionList(panel, sessions);
            return;
        }
    } catch (err) {
        console.warn('[HistoryList] get_execution_sessions failed, falling back:', err);
    }
    // Fallback to legacy get_history
    await loadLegacyHistory(panel);
}

/**
 * Render session list (new session-based UI)
 * @param {HTMLElement} panel - The panel element
 * @param {Array} sessions - Array of execution sessions
 */
function renderSessionList(panel, sessions) {
    const historyItems = panel.querySelector('#history-items');
    if (!historyItems) return;

    if (sessions.length === 0) {
        historyItems.innerHTML = '<div class="empty-state">No execution sessions yet</div>';
        return;
    }

    historyItems.innerHTML = sessions.map((session, index) => {
        const time = formatTime(session.started_at);
        const statusClass = `level-${session.status}`;
        const summary = truncate(session.task_summary || 'unknown task', 40);

        return `
            <div class="history-item session-item" data-index="${index}" data-session-id="${escapeHtml(session.session_id)}">
                <span class="col-time">${time}</span>
                <span class="col-level"><span class="level-tag ${statusClass}">${escapeHtml(session.status)}</span></span>
                <span class="col-title" title="${escapeHtml(session.task_summary)}">${escapeHtml(summary)}</span>
            </div>
        `;
    }).join('');

    // Bind click events
    historyItems.querySelectorAll('.session-item').forEach((item) => {
        item.addEventListener('click', async () => {
            const sessionId = item.dataset.sessionId;
            await showSessionDetail(panel, sessionId);

            historyItems.querySelectorAll('.history-item').forEach(i => i.classList.remove('active'));
            item.classList.add('active');
        });
    });
}

/**
 * Show session detail with progress events
 * @param {HTMLElement} panel - The panel element
 * @param {string} sessionId - The session ID
 */
async function showSessionDetail(panel, sessionId) {
    const detailContent = panel.querySelector('#detail-content');
    if (!detailContent) return;

    try {
        const progress = await invoke('get_session_progress', { sessionId });

        if (progress && progress.length > 0) {
            detailContent.innerHTML = `
                <div class="session-detail">
                    <div class="session-progress-list">
                        ${progress.map(event => {
                            const status = event.status_label || event.level || 'info';
                            return `
                            <div class="progress-item">
                                <div class="progress-meta">
                                    <span class="progress-level level-${escapeHtml(status.toLowerCase())}">${escapeHtml(event.event_type)}</span>
                                    <span>${formatTime(event.timestamp)}</span>
                                </div>
                                <div class="progress-message">${escapeHtml(event.message || '-')}</div>
                            </div>
                        `}).join('')}
                    </div>
                </div>
            `;
        } else {
            detailContent.innerHTML = '<div class="empty-state">No progress events for this session</div>';
        }
    } catch (err) {
        console.error('[HistoryList] Failed to load session progress:', err);
        detailContent.innerHTML = '<div class="empty-state">Failed to load session details</div>';
    }
}

/**
 * Load legacy history (fallback when session API unavailable)
 * @param {HTMLElement} panel - The panel element
 */
async function loadLegacyHistory(panel) {
    try {
        const records = await invoke('get_history', { limit: 50 });
        const historyItems = panel.querySelector('#history-items');
        if (!historyItems) return;

        if (!records || records.length === 0) {
            historyItems.innerHTML = '<div class="empty-state">No history available</div>';
            return;
        }

        // Sort by timestamp descending (newest first)
        const sortedRecords = [...records].sort((a, b) => {
            const timeA = a.timestamp || a.time || 0;
            const timeB = b.timestamp || b.time || 0;
            return timeB - timeA;
        });

        historyItems.innerHTML = sortedRecords.map((record, index) => {
            const time = formatTime(record.timestamp || record.time);
            const status = record.status_label || record.level || 'suggest';
            const title = record.windowTitle || record.window_title || record.title || '-';
            const levelClass = `level-${status.toLowerCase()}`;
            const levelDisplay = status.toUpperCase();

            return `
                <div class="history-item" data-index="${index}">
                    <span class="col-time">${time}</span>
                    <span class="col-level"><span class="level-tag ${levelClass}">${levelDisplay}</span></span>
                    <span class="col-title" title="${escapeHtml(title)}">${escapeHtml(truncate(title, 30))}</span>
                </div>
            `;
        }).join('');

        // Bind click events to history items
        historyItems.querySelectorAll('.history-item').forEach((item) => {
            item.addEventListener('click', () => {
                const index = parseInt(item.dataset.index, 10);
                showHistoryDetail(panel, sortedRecords[index]);

                // Update active state
                historyItems.querySelectorAll('.history-item').forEach(i => i.classList.remove('active'));
                item.classList.add('active');
            });
        });

    } catch (err) {
        console.error('Failed to load history:', err);
        const historyItems = panel.querySelector('#history-items');
        if (historyItems) {
            historyItems.innerHTML = '<div class="empty-state">Failed to load history</div>';
        }
    }
}

/**
 * Show detail view for a history record
 * @param {HTMLElement} panel - The panel element
 * @param {Object} record - The history record
 */
function showHistoryDetail(panel, record) {
    const detailContent = panel.querySelector('#detail-content');
    if (!detailContent || !record) return;

    const status = record.status_label || record.level || 'suggest';
    const levelClass = `level-${status.toLowerCase()}`;
    const levelDisplay = status.toUpperCase();
    const modelText = record.model_text || record.nextStep || record.next_step || '-';
    const confidenceScore = record.confidence_score ?? record.confidence ?? 0;

    detailContent.innerHTML = `
        <div class="judgment-block">
            <div class="judgment-line">
                <span class="judgment-key">judgment:</span>
                <span class="judgment-val">${escapeHtml(record.judgment || '-')}</span>
            </div>
            <div class="judgment-line">
                <span class="judgment-key">model_text:</span>
                <span class="judgment-val">${escapeHtml(modelText)}</span>
            </div>
            <div class="judgment-line">
                <span class="judgment-key">status_label:</span>
                <span class="level-tag ${levelClass}">${levelDisplay}</span>
            </div>
            <div class="judgment-line">
                <span class="judgment-key">confidence_score:</span>
                <span class="judgment-val">${Number(confidenceScore).toFixed(2)}</span>
            </div>
            <div class="judgment-line">
                <span class="judgment-key">grounds:</span>
                <span class="judgment-val">${escapeHtml(record.grounds || '-')}</span>
            </div>
            <div class="judgment-line">
                <span class="judgment-key">action:</span>
                <span class="judgment-val">${escapeHtml(record.system_action || record.action || '-')}</span>
            </div>
            <div class="judgment-line">
                <span class="judgment-key">time:</span>
                <span class="judgment-val">${formatDateTime(record.timestamp || record.time)}</span>
            </div>
        </div>
    `;
}

/**
 * Format timestamp to time string (HH:MM:SS)
 * @param {number|string} timestamp - Unix timestamp
 * @returns {string} Formatted time string
 */
function formatTime(timestamp) {
    if (!timestamp) return '--:--:--';
    const date = new Date(timestamp * 1000);
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    const seconds = String(date.getSeconds()).padStart(2, '0');
    return `${hours}:${minutes}:${seconds}`;
}

/**
 * Format timestamp to full datetime string
 * @param {number|string} timestamp - Unix timestamp
 * @returns {string} Formatted datetime string
 */
function formatDateTime(timestamp) {
    if (!timestamp) return '-';
    const date = new Date(timestamp * 1000);
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    const seconds = String(date.getSeconds()).padStart(2, '0');
    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

/**
 * Escape HTML special characters
 * @param {string} text - Text to escape
 * @returns {string} Escaped text
 */
function escapeHtml(text) {
    if (typeof text !== 'string') return text || '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

/**
 * Truncate text to specified length
 * @param {string} text - Text to truncate
 * @param {number} maxLength - Maximum length
 * @returns {string} Truncated text
 */
function truncate(text, maxLength) {
    if (!text || text.length <= maxLength) return text;
    return text.substring(0, maxLength) + '...';
}
