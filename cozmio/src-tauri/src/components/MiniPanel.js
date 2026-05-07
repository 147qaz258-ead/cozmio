/**
 * MiniPanel - floating mini window UI component.
 * Subscribes to state-update events and renders a simplified status view.
 */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let currentState = null;
let unlistenFn = null;

// State color mapping from backend state to UI color name
const STATE_COLORS = {
  idle: '#636366',
  monitoring: '#0a84ff',
  analyzing: '#0a84ff',
  confirm: '#ff9f0a',
  executing: '#ff9f0a',
  done: '#34c759',
  error: '#ff3b30',
};

function computeMiniState(state) {
  const trayState = state.tray_state || 'idle';
  const runningState = state.running_state || 'Stopped';
  const pendingConfirmation = state.pending_confirmation;
  const relayExecution = state.relay_execution;

  // Priority: error > confirm > executing > analyzing > monitoring > idle
  if (relayExecution) {
    const status = relayExecution.relay_status || '';
    if (status === 'failed' || status === 'error') return 'error';
    if (status === 'completed' || status === 'done') return 'done';
    if (['running', 'waiting', 'dispatching'].includes(status)) return 'executing';
  }

  if (pendingConfirmation) return 'confirm';

  if (trayState === 'processing') return 'analyzing';

  if (runningState === 'Running') return 'monitoring';

  return 'idle';
}

function stateLabel(miniState) {
  const map = {
    idle: '空闲中',
    monitoring: '监控中',
    analyzing: '分析中',
    confirm: '待确认',
    executing: '执行中',
    done: '已完成',
    error: '错误',
  };
  return map[miniState] || '未知';
}

function stateSubtitle(miniState) {
  const map = {
    idle: '工位空闲',
    monitoring: '正在监控前台窗口',
    analyzing: '正在分析窗口内容',
    confirm: '需要您的确认',
    executing: '任务执行中',
    done: '执行完成',
    error: '发生错误',
  };
  return map[miniState] || '';
}

// Progress ring SVG with the given color
function progressRingSVG(color, isDone) {
  if (isDone) {
    // Checkmark circle
    return `<svg viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="20" cy="20" r="16" stroke="${color}" stroke-width="3" fill="none"/>
      <path d="M13 20l5 5 9-10" stroke="${color}" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>`;
  }
  // Default ring
  return `<svg viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="20" cy="20" r="16" stroke="${color}" stroke-width="3" fill="none" opacity="0.3"/>
    <circle cx="20" cy="20" r="16" stroke="${color}" stroke-width="3" fill="none"
      stroke-dasharray="100.5" stroke-dashoffset="25" stroke-linecap="round"
      transform="rotate(-90 20 20)"/>
  </svg>`;
}

function miniDotSVG(color) {
  return `<svg width="8" height="8" viewBox="0 0 8 8">
    <circle cx="4" cy="4" r="4" fill="${color}"/>
  </svg>`;
}

async function handleMiniAction(action) {
  try {
    await invoke('mini_action', { action });
  } catch (err) {
    console.error(`[MiniPanel] mini_action(${action}) failed:`, err);
  }
}

function renderMiniPanel(state) {
  currentState = state;
  const panel = document.getElementById('mini-panel');
  if (!panel) return;

  const miniState = computeMiniState(state);
  const color = STATE_COLORS[miniState] || '#636366';
  const label = stateLabel(miniState);
  const subtitle = stateSubtitle(miniState);

  // Build inner HTML
  let html = '';

  // === Status row: ring + name ===
  html += `<div class="status-row">`;
  html += `<div class="status-ring">${progressRingSVG(color, miniState === 'done')}</div>`;
  html += `<div>`;
  html += `<div class="status-name">${escapeHtml(label)}</div>`;
  html += `<div class="status-subtitle">${escapeHtml(subtitle)}</div>`;
  html += `</div>`;
  html += `</div>`;

  // === Main card ===
  const mainDotColor = color;
  const mainTitle = mainCardTitle(miniState, state);
  const mainDesc = mainCardDesc(miniState, state);

  html += `<div class="main-card">`;
  html += `<div class="main-card-header">`;
  html += `<div class="main-card-dot" style="background:${mainDotColor}"></div>`;
  html += `<div class="main-card-title">${escapeHtml(mainTitle)}</div>`;
  html += `</div>`;
  html += `<div class="main-card-desc">${escapeHtml(mainDesc)}</div>`;
  html += `<div class="main-card-actions">${renderMainActions(miniState, state)}</div>`;
  html += `</div>`;

  // === Task card (if applicable) ===
  const taskCard = renderTaskCard(miniState, state);
  if (taskCard) {
    html += taskCard;
  }

  // === Progress bar (during analyzing/executing/confirm) ===
  if (['analyzing', 'executing', 'confirm'].includes(miniState)) {
    const progressPercent = computeProgress(state);
    html += `<div class="progress-section">`;
    html += `<div class="progress-label">${escapeHtml(progressLabel(miniState))}</div>`;
    html += `<div class="progress-bar-bg">`;
    html += `<div class="progress-bar-fill" style="width:${progressPercent}%"></div>`;
    html += `</div>`;
    html += `</div>`;
  }

  // === Bottom dots: Ollama + Relay status ===
  html += `<div class="bottom-dots">`;
  html += `<div class="status-dot ${ollamaDotClass(state)}"></div>`;
  html += `<span class="dot-label">Ollama</span>`;
  html += `<div class="status-dot ${relayDotClass(state)}"></div>`;
  html += `<span class="dot-label">Relay</span>`;
  html += `</div>`;

  panel.innerHTML = html;

  // Attach event listeners on action buttons
  panel.querySelectorAll('[data-action]').forEach(btn => {
    btn.addEventListener('click', () => {
      const action = btn.dataset.action;
      handleMiniAction(action);
    });
  });
}

function mainCardTitle(miniState, state) {
  if (miniState === 'idle') return '空闲中';
  if (miniState === 'monitoring') return '监控中';
  if (miniState === 'analyzing') return '分析中';
  if (miniState === 'confirm') return '待确认';
  if (miniState === 'executing') return '执行中';
  if (miniState === 'done') return '已完成';
  if (miniState === 'error') {
    return state.relay_execution?.error_message ? '执行错误' : '发生错误';
  }
  return '工位空闲';
}

function mainCardDesc(miniState, state) {
  if (miniState === 'idle') return '工位空闲，随时准备监控';
  if (miniState === 'monitoring') {
    const win = state.current_window;
    return win?.title ? `监控中: ${win.title}` : '正在监控前台窗口';
  }
  if (miniState === 'analyzing') return '正在分析窗口内容...';
  if (miniState === 'confirm') {
    const pending = state.pending_confirmation;
    return pending?.task_text || '有待确认的任务';
  }
  if (miniState === 'executing') {
    const task = state.current_task;
    return task?.task_text || '任务执行中...';
  }
  if (miniState === 'done') {
    const relay = state.relay_execution;
    return relay?.result_summary || '执行已完成';
  }
  if (miniState === 'error') {
    const relay = state.relay_execution;
    return relay?.error_message || '发生未知错误';
  }
  return '';
}

function renderMainActions(miniState, state) {
  if (miniState === 'idle') {
    return `<button class="btn-mini btn-primary" data-action="toggle">开始监控</button>`;
  }
  if (miniState === 'monitoring') {
    return `<button class="btn-mini btn-secondary" data-action="toggle">暂停</button>`;
  }
  if (miniState === 'confirm') {
    return `<button class="btn-mini btn-warning" data-action="confirm">去确认</button>`;
  }
  if (miniState === 'executing') {
    return `<button class="btn-mini btn-danger" data-action="interrupt">中断</button>`;
  }
  if (miniState === 'done') {
    return `<button class="btn-mini btn-secondary" data-action="toggle">查看</button>`;
  }
  if (miniState === 'error') {
    return `<button class="btn-mini btn-primary" data-action="toggle">重试</button>`;
  }
  return '';
}

function renderTaskCard(miniState, state) {
  const pending = state.pending_confirmation;
  const task = state.current_task;

  if (miniState === 'confirm' && pending) {
    return `<div class="task-card">
      <div class="task-label">待确认任务</div>
      <div class="task-title">${escapeHtml(pending.task_text || '-')}</div>
      <div class="task-source">${escapeHtml(pending.source_window || '-')} · ${escapeHtml(pending.source_process || '-')}</div>
    </div>`;
  }

  if (miniState === 'executing' && task) {
    return `<div class="task-card">
      <div class="task-label">当前任务</div>
      <div class="task-title">${escapeHtml(task.task_text || '-')}</div>
      <div class="task-source">${escapeHtml(task.source_window || '-')}</div>
    </div>`;
  }

  return '';
}

function computeProgress(state) {
  const relay = state.relay_execution;
  if (!relay) return 0;
  const progress = relay.progress || [];
  if (progress.length === 0) return 30; // default in-progress feel
  return Math.min(90, 30 + progress.length * 10);
}

function progressLabel(miniState) {
  if (miniState === 'analyzing') return '分析进度';
  if (miniState === 'executing') return '执行进度';
  if (miniState === 'confirm') return '等待确认';
  return '进度';
}

function ollamaDotClass(state) {
  // Ollama is "connected" if we have recent judgments or model_name is set
  if (state.model_name) return 'blue';
  return 'gray';
}

function relayDotClass(state) {
  const relay = state.relay_execution;
  if (!relay) return 'gray';
  const status = relay.relay_status || '';
  if (['running', 'waiting', 'dispatching'].includes(status)) return 'orange';
  if (status === 'completed' || status === 'done') return 'green';
  if (status === 'failed' || status === 'error') return 'red';
  return 'gray';
}

function escapeHtml(text) {
  if (typeof text !== 'string') return '';
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Initialize: listen for state-update events
async function init() {
  unlistenFn = await listen('state-update', (event) => {
    renderMiniPanel(event.payload || {});
  });

  // Also fetch initial state
  try {
    const { get_ui_state } = window.__TAURI__.core;
    const initialState = await get_ui_state();
    renderMiniPanel(initialState);
  } catch (err) {
    console.error('[MiniPanel] Failed to get initial state:', err);
  }

  // Clean up listener when window is unloaded
  window.addEventListener('beforeunload', () => {
    if (unlistenFn) {
      unlistenFn();
      unlistenFn = null;
    }
  });
}

init().catch(console.error);