/**
 * StatusPanel - task monitor view for pending confirmations and relay execution.
 */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

import {
    agentImageForVisualState,
    agentVisualState,
    buildExecutionPresence,
    compactAgeLabel,
    pickPrimaryTarget,
    presenceStatusLabel,
    presenceToneClass
} from './PresenceModel.js';

export function createStatusPanel(state) {
    const panel = document.createElement('div');
    panel.id = 'panel-status';
    panel.className = 'content-panel';

    // Track update banner visibility
    panel.updatePendingVersion = null;

    panel.addEventListener('click', async (event) => {
        const action = event.target.closest('[data-action]')?.dataset.action;
        if (!action) {
            return;
        }

        try {
            if (action === 'toggle-running') {
                const currentState = panel.dataset.state || 'Stopped';
                if (currentState === 'Stopped') {
                    await invoke('start_running');
                } else {
                    await invoke('stop_running');
                }
            } else if (action === 'interrupt-task') {
                await invoke('interrupt_current_task');
            } else if (action === 'restart-application') {
                await invoke('restart_application');
            } else if (action === 'dismiss-update-banner') {
                await invoke('dismiss_update_reminder');
                panel.updatePendingVersion = null;
                updateStatusPanel(panel, { ...getCurrentPanelState(panel) });
            }
        } catch (err) {
            console.error(`Failed to run action ${action}:`, err);
        }
    });

    updateStatusPanel(panel, state || {});
    return panel;
}

function getCurrentPanelState(panel) {
    // Return current state including updatePendingVersion
    return {
        updatePendingVersion: panel.updatePendingVersion,
        // Include other necessary state fields
    };
}

export function updateStatusPanel(panel, state) {
    if (!panel) return;

    // Preserve updatePendingVersion if not provided in new state
    const updatePendingVersion = state.updatePendingVersion !== undefined
        ? state.updatePendingVersion
        : panel.updatePendingVersion;

    const mergedState = {
        agentState: 'Stopped',
        interval: 3,
        ollamaUrl: 'localhost:11434',
        modelName: 'llava',
        windowInfo: null,
        pendingConfirmation: null,
        currentTask: null,
        relayExecution: null,
        updatePendingVersion: updatePendingVersion,
        ...state
    };

    panel.updatePendingVersion = updatePendingVersion;
    panel.dataset.state = mergedState.agentState || 'Stopped';
    panel.innerHTML = renderPanel(mergedState);
}

function renderPanel(state) {
    const presence = buildExecutionPresence({
        ...state,
        running_state: state.agentState,
        relay_execution: state.relayExecution,
        current_task: state.currentTask,
        pending_confirmation: state.pendingConfirmation,
        poll_interval_secs: state.interval
    });

    return `
        <div class="panel-title">WORKSTATION</div>

        ${renderWorkstationHero(state, presence)}
        ${renderExecutionPresence(presence)}
        ${renderCurrentHandoff(state, presence)}
        ${renderUpdateBanner(state.updatePendingVersion)}

        <div class="panel-title panel-title-secondary">DETAILS</div>

        <div class="module-card">
            <div class="module-header">
                <span class="module-name">AGENT STATE</span>
                <span class="module-tag">${escapeHtml(state.agentState)}</span>
            </div>
            <div class="monitor-meta">
                <div class="monitor-pill">
                    <span class="status-dot ${statusDotClass(state.agentState)}"></span>
                    <span>${escapeHtml((state.agentState || 'Stopped').toLowerCase())}</span>
                </div>
                <div class="monitor-pill">interval ${escapeHtml(String(state.interval || 3))}s</div>
                <div class="monitor-pill">${escapeHtml(state.ollamaUrl || '-')}</div>
                <div class="monitor-pill">${escapeHtml(state.modelName || '-')}</div>
                ${state.inferenceSource ? `<div class="monitor-pill inference-source">${escapeHtml(state.inferenceSource)}</div>` : ''}
            </div>
            <div class="btn-group monitor-actions">
                <button class="btn btn-primary" data-action="toggle-running">
                    ${state.agentState === 'Stopped' ? 'START' : 'STOP'}
                </button>
            </div>
        </div>

        <div class="module-card">
            <div class="module-header">
                <span class="module-name">LAST JUDGMENT</span>
                <span class="module-tag">${escapeHtml(state.lastJudgment?.judgment || '-')}</span>
            </div>
            ${renderLastJudgment(state.lastJudgment, state.windowInfo)}
        </div>

        <div class="module-card">
            <div class="module-header">
                <span class="module-name">CURRENT TASK</span>
                <span class="module-tag">${escapeHtml(currentTaskStateLabel(state.currentTask, state.relayExecution))}</span>
            </div>
            ${renderCurrentTask(state.currentTask, state.windowInfo)}
        </div>

        <div class="module-card">
            <div class="module-header">
                <span class="module-name">RELAY SESSION</span>
                <span class="module-tag">${escapeHtml(state.relayExecution?.relay_status || 'idle')}</span>
            </div>
            ${renderRelaySession(state.relayExecution)}
        </div>

        <div class="module-card">
            <div class="module-header">
                <span class="module-name">RECENT PROGRESS</span>
            </div>
            ${renderProgress(state.relayExecution)}
        </div>

        <div class="module-card">
            <div class="module-header">
                <span class="module-name">RESULT</span>
            </div>
            ${renderResult(state.relayExecution)}
        </div>
    `;
}

function renderWorkstationHero(state, presence) {
    const primary = pickPrimaryTarget(presence);
    const tone = presenceToneClass(presence.overall_status);
    const visualState = agentVisualState({
        ...state,
        running_state: state.agentState,
        relay_execution: state.relayExecution,
        current_task: state.currentTask,
        pending_confirmation: state.pendingConfirmation
    }, presence);
    const imageSrc = agentImageForVisualState(visualState);
    const activeCount = presence.targets.filter((target) => ['online', 'working'].includes(target.status)).length;
    const totalCount = presence.targets.length;

    return `
        <section class="workstation-hero ${tone}" data-visual-state="${escapeHtml(visualState)}">
            <div class="pixel-workstation" aria-hidden="true">
                <img class="workstation-agent-img" src="${escapeHtml(imageSrc)}" alt="">
            </div>
            <div class="workstation-copy">
                <div class="workstation-kicker">EXECUTION PRESENCE</div>
                <div class="workstation-title">${escapeHtml(presenceStatusLabel(presence.overall_status))} · ${activeCount}/${totalCount} 在岗</div>
                <div class="workstation-summary">${escapeHtml(presence.summary)}</div>
                <div class="workstation-meta">
                    <span>${escapeHtml(primary?.label || '执行端')}</span>
                    <span>${escapeHtml(primary?.detail || '等待状态更新')}</span>
                    ${state.inferenceSource ? `<span>${escapeHtml(state.inferenceSource)}</span>` : ''}
                </div>
            </div>
        </section>
    `;
}

function renderExecutionPresence(presence) {
    return `
        <section class="presence-section">
            <div class="section-label">EXECUTION TARGETS</div>
            <div class="presence-grid">
                ${presence.targets.map((target) => renderPresenceCard(target)).join('')}
            </div>
        </section>
    `;
}

function renderPresenceCard(target) {
    const tone = presenceToneClass(target.status);
    return `
        <article class="presence-card ${tone}">
            <div class="presence-card-top">
                <span class="presence-led"></span>
                <span class="presence-kind">${escapeHtml(target.kind)}</span>
            </div>
            <div class="presence-name">${escapeHtml(target.label)}</div>
            <div class="presence-status">${escapeHtml(presenceStatusLabel(target.status))}</div>
            <div class="presence-detail">${escapeHtml(target.detail)}</div>
            <div class="presence-foot">
                <span>${escapeHtml(compactAgeLabel(target.heartbeat_age_secs))}</span>
                ${target.session_id ? `<span class="presence-session">${escapeHtml(shortId(target.session_id))}</span>` : '<span>standby</span>'}
            </div>
            ${target.task ? `<div class="presence-task">${escapeHtml(target.task)}</div>` : ''}
        </article>
    `;
}

function renderCurrentHandoff(state, presence) {
    const pending = state.pendingConfirmation;
    const task = state.currentTask;
    const relay = state.relayExecution;
    const primary = pickPrimaryTarget(presence);

    let title = '没有活跃交接';
    let body = '执行端待机，当前没有需要确认或正在执行的任务。';

    if (pending) {
        title = '等待确认';
        body = pending.task_text || pending.taskText || '有任务需要确认。';
    } else if (task) {
        title = '当前任务';
        body = task.task_text || task.taskText || '任务正在执行链路中。';
    } else if (relay?.result_summary) {
        title = '最近结果';
        body = relay.result_summary;
    } else if (primary?.status === 'degraded') {
        title = '链路异常';
        body = primary.detail;
    }

    return `
        <section class="handoff-card ${presenceToneClass(presence.overall_status)}">
            <div>
                <div class="section-label">CURRENT HANDOFF</div>
                <div class="handoff-title">${escapeHtml(title)}</div>
                <div class="handoff-body">${escapeHtml(body)}</div>
            </div>
            ${isInterruptible(relay) ? `
                <button class="btn btn-danger" data-action="interrupt-task">STOP</button>
            ` : ''}
        </section>
    `;
}

function renderCurrentTask(currentTask, windowInfo) {
    if (!currentTask) {
        return `
            <div class="empty-state">No active task</div>
            <div class="monitor-grid">
                <div class="monitor-cell">
                    <div class="monitor-label">foreground window</div>
                    <div class="monitor-value">${escapeHtml(windowInfo?.title || '-')}</div>
                </div>
                <div class="monitor-cell">
                    <div class="monitor-label">process</div>
                    <div class="monitor-value">${escapeHtml(windowInfo?.processName || windowInfo?.process || '-')}</div>
                </div>
            </div>
        `;
    }

    return `
        <div class="task-text-block">${escapeHtml(currentTask.taskText || currentTask.task_text || '-')}</div>
        <div class="monitor-grid">
            <div class="monitor-cell">
                <div class="monitor-label">source window</div>
                <div class="monitor-value">${escapeHtml(currentTask.sourceWindow || currentTask.source_window || '-')}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">source process</div>
                <div class="monitor-value">${escapeHtml(currentTask.sourceProcess || currentTask.source_process || '-')}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">created</div>
                <div class="monitor-value">${escapeHtml(formatTimestamp(currentTask.createdAt || currentTask.created_at))}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">task state</div>
                <div class="monitor-value">${escapeHtml(currentTask.taskState || currentTask.task_state || '-')}</div>
            </div>
        </div>
    `;
}

function renderRelaySession(relayExecution) {
    if (!relayExecution) {
        return `<div class="empty-state">No relay session yet</div>`;
    }

    return `
        <div class="monitor-grid">
            <div class="monitor-cell">
                <div class="monitor-label">transport</div>
                <div class="monitor-value">${escapeHtml(relayExecution.transport || '-')}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">session id</div>
                <div class="monitor-value monitor-break">${escapeHtml(relayExecution.session_id || '-')}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">status</div>
                <div class="monitor-value">${escapeHtml(relayExecution.relay_status || '-')}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">updated</div>
                <div class="monitor-value">${escapeHtml(formatTimestamp(relayExecution.updated_at))}</div>
            </div>
        </div>
        ${isInterruptible(relayExecution) ? `
            <div class="btn-group monitor-actions">
                <button class="btn btn-danger" data-action="interrupt-task">STOP</button>
            </div>
        ` : ''}
    `;
}

function renderProgress(relayExecution) {
    const progressItems = Array.isArray(relayExecution?.progress) ? relayExecution.progress.slice(-8) : [];
    if (progressItems.length === 0) {
        return `<div class="empty-state">No relay progress yet</div>`;
    }

    return `
        <div class="progress-list">
            ${progressItems.map((item) => {
                const status = item.status_label || item.level || 'info';
                return `
                <div class="progress-item">
                    <div class="progress-meta">
                        <span class="progress-level level-${escapeHtml(status.toLowerCase())}">${escapeHtml(status.toUpperCase())}</span>
                        <span>${escapeHtml(formatTimestamp(item.timestamp))}</span>
                    </div>
                    <div class="progress-message">${escapeHtml(item.message || '-')}</div>
                </div>
            `}).join('')}
        </div>
    `;
}

function renderResult(relayExecution) {
    if (!relayExecution) {
        return `<div class="empty-state">No result yet</div>`;
    }

    return `
        <div class="monitor-stack">
            <div class="monitor-cell">
                <div class="monitor-label">summary</div>
                <div class="monitor-value">${escapeHtml(relayExecution.result_summary || '-')}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">error</div>
                <div class="monitor-value ${relayExecution.error_message ? 'monitor-error' : ''}">${escapeHtml(relayExecution.error_message || '-')}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">output</div>
                <pre class="result-output">${escapeHtml(relayExecution.result_output || '-')}</pre>
            </div>
        </div>
    `;
}

function renderLastJudgment(lastJudgment, windowInfo) {
    if (!lastJudgment) {
        return `
            <div class="empty-state">No judgment yet</div>
            <div class="monitor-grid">
                <div class="monitor-cell">
                    <div class="monitor-label">foreground window</div>
                    <div class="monitor-value">${escapeHtml(windowInfo?.title || '-')}</div>
                </div>
                <div class="monitor-cell">
                    <div class="monitor-label">process</div>
                    <div class="monitor-value">${escapeHtml(windowInfo?.processName || windowInfo?.process || '-')}</div>
                </div>
            </div>
        `;
    }

    const pc = lastJudgment.process_context;

    const modelText = lastJudgment.model_text || lastJudgment.next_step || '-';
    const confidenceScore = lastJudgment.confidence_score ?? lastJudgment.confidence;

    return `
        <div class="task-text-block">${escapeHtml(modelText)}</div>
        <div class="monitor-grid">
            <div class="monitor-cell">
                <div class="monitor-label">judgment</div>
                <div class="monitor-value">${escapeHtml(lastJudgment.judgment || '-')}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">confidence</div>
                <div class="monitor-value">${confidenceScore != null ? Number(confidenceScore).toFixed(2) : '-'}</div>
            </div>
            ${pc ? `
            <div class="monitor-cell">
                <div class="monitor-label">stay duration</div>
                <div class="monitor-value">${escapeHtml(pc.stay_duration_seconds)}s</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">switches/min</div>
                <div class="monitor-value">${escapeHtml(pc.switches_in_last_minute)}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">rapid switches &lt;5s</div>
                <div class="monitor-value">${escapeHtml(pc.rapid_switch_intervals_under_5s ?? 0)}</div>
            </div>
            <div class="monitor-cell">
                <div class="monitor-label">changed &lt;5s</div>
                <div class="monitor-value">${pc.foreground_changed_within_5s ? 'YES' : 'no'}</div>
            </div>
            ` : ''}
        </div>
    `;
}

function currentTaskStateLabel(currentTask, relayExecution) {
    if (currentTask?.taskState || currentTask?.task_state) {
        return currentTask.taskState || currentTask.task_state;
    }
    return relayExecution?.relay_status || 'idle';
}

function statusDotClass(agentState) {
    if (agentState === 'Stopped') {
        return 'paused';
    }
    if (agentState === 'Processing') {
        return 'processing';
    }
    return '';
}

function isInterruptible(relayExecution) {
    return ['connecting', 'dispatching', 'running', 'waiting', 'interrupting'].includes(
        relayExecution?.relay_status || ''
    );
}

function escapeHtml(text) {
    if (typeof text !== 'string') return text;
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatTimestamp(unixSeconds) {
    if (!unixSeconds) return '-';
    const date = new Date(unixSeconds * 1000);
    if (Number.isNaN(date.getTime())) return '-';
    return date.toLocaleString();
}

function renderUpdateBanner(updatePendingVersion) {
    if (!updatePendingVersion) {
        return '';
    }
    return `
        <div class="update-pending-banner">
            <span>Update available: ${escapeHtml(updatePendingVersion)}</span>
            <button class="btn-restart" data-action="restart-application">立即重启</button>
            <button class="btn-later" data-action="dismiss-update-banner">稍后</button>
        </div>
    `;
}

function shortId(value) {
    if (typeof value !== 'string') return '-';
    if (value.length <= 10) return value;
    return `${value.slice(0, 6)}...${value.slice(-4)}`;
}

export async function initStateUpdateListener(panel) {
    const unlisten = await listen('state-update', (event) => {
        updateStatusPanel(panel, transformBackendState(event.payload));
    });

    const unlistenRunning = await listen('running-state-changed', (event) => {
        updateStatusPanel(panel, { agentState: event.payload || 'Stopped' });
    });

    const unlistenUpdateReady = await listen('update-ready', (event) => {
        const payload = event.payload;
        panel.updatePendingVersion = typeof payload === 'string'
            ? payload
            : (payload?.version || 'new version');
        updateStatusPanel(panel, { updatePendingVersion: panel.updatePendingVersion });
    });

    return () => {
        unlisten();
        unlistenRunning();
        unlistenUpdateReady();
    };
}

export async function fetchAndApplyRunningState(panel) {
    try {
        const uiState = await invoke('get_ui_state');
        updateStatusPanel(panel, transformBackendState(uiState));
    } catch (err) {
        console.error('Failed to fetch UI state:', err);
        try {
            const runningState = await invoke('get_running_state');
            updateStatusPanel(panel, { agentState: runningState });
        } catch (fallbackErr) {
            console.error('Failed to fetch running state:', fallbackErr);
        }
    }
}

function transformBackendState(payload) {
    const agentState = payload.tray_state === 'processing'
        ? 'Processing'
        : (payload.running_state || 'Stopped');

    return {
        agentState,
        interval: payload.poll_interval_secs,
        ollamaUrl: payload.ollama_url,
        modelName: payload.model_name,
        windowInfo: payload.current_window ? {
            title: payload.current_window.title || '-',
            processName: payload.current_window.process_name || '-',
            processId: payload.current_window.process_id,
            monitorIndex: payload.current_window.monitor_index
        } : null,
        lastJudgment: payload.last_judgment || null,
        pendingConfirmation: payload.pending_confirmation || null,
        currentTask: payload.current_task || null,
        relayExecution: payload.relay_execution || null,
        inferenceSource: payload.inference_source || null
    };
}
