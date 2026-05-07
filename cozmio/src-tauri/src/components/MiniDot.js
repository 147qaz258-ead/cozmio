const tauriCore = window.__TAURI__ && window.__TAURI__.core;
const tauriEvent = window.__TAURI__ && window.__TAURI__.event;
const invoke = tauriCore && tauriCore.invoke;
const listen = tauriEvent && tauriEvent.listen;

const ACTIVE_RELAY_STATUSES = new Set(['connecting', 'dispatching', 'running', 'waiting', 'interrupting']);
const ATTENTION_RELAY_STATUSES = new Set(['relay_unavailable', 'subscription_error', 'interrupt_error']);
const FAILED_RELAY_STATUSES = new Set(['failed', 'error', 'dispatch_error']);
const DONE_RELAY_STATUSES = new Set(['completed', 'done', 'interrupted']);

const PRESENCE_LABELS = {
  online: '在线',
  working: '工作中',
  attention: '待处理',
  degraded: '异常',
  offline: '离线',
  unknown: '待机'
};

const PRESENCE_COLORS = {
  online: '#34c759',
  working: '#0a84ff',
  attention: '#ff9f0a',
  degraded: '#ff3b30',
  offline: '#cc6a6a',
  unknown: '#8e8e93'
};

const STATUS_PRIORITY = {
  degraded: 5,
  offline: 4,
  attention: 3,
  working: 2,
  online: 1,
  unknown: 0
};

let currentState = {};

async function handleAction(action) {
  try {
    if (invoke) await invoke('mini_action', { action });
  } catch (err) {
    console.error('[MiniWorkstation] mini_action failed:', err);
  }
}

function buildExecutionPresence(state) {
  const now = Math.floor(Date.now() / 1000);
  const runningState = state.running_state || state.agentState || 'Stopped';
  const trayState = state.tray_state || '';
  const relayExecution = state.relay_execution || state.relayExecution || null;
  const currentTask = state.current_task || state.currentTask || null;
  const pendingConfirmation = state.pending_confirmation || state.pendingConfirmation || null;
  const relayStatus = relayExecution && relayExecution.relay_status || '';
  const relayUpdatedAt = relayExecution && relayExecution.updated_at || null;
  const sessionId = relayExecution && relayExecution.session_id || null;
  const taskText = taskTextFrom(currentTask, pendingConfirmation);

  const targets = [
    {
      id: 'desktop-host', label: '桌面宿主', kind: 'host', status: 'online',
      detail: '主进程正在上报状态', heartbeat_age_secs: 0, session_id: null, task: null
    },
    {
      id: 'monitor-loop', label: '观察循环', kind: 'monitor',
      status: trayState === 'processing' ? 'working' : runningState === 'Running' ? 'online' : 'unknown',
      detail: trayState === 'processing' ? '正在分析前台窗口' : runningState === 'Running' ? `以 ${state.poll_interval_secs || state.interval || 3}s 间隔观察` : '监控循环已停止',
      heartbeat_age_secs: null, session_id: null, task: null
    },
    buildRelayTarget(relayExecution, relayStatus, relayUpdatedAt, now),
    buildClaudeTarget(relayExecution, relayStatus, sessionId, taskText, relayUpdatedAt, now)
  ];

  const overallStatus = pendingConfirmation ? 'attention' : targets.reduce((highest, target) => {
    return STATUS_PRIORITY[target.status] > STATUS_PRIORITY[highest] ? target.status : highest;
  }, 'unknown');

  return {
    overall_status: overallStatus,
    summary: summaryFor(overallStatus, targets, pendingConfirmation, relayExecution),
    targets,
    updated_at: now
  };
}

function buildRelayTarget(relayExecution, relayStatus, updatedAt, now) {
  if (!relayExecution) {
    return { id: 'relay-engine', label: 'Relay Engine', kind: 'relay', status: 'unknown', detail: '尚无接力会话，等待首次派发', heartbeat_age_secs: null, session_id: null, task: null };
  }
  let status = 'unknown';
  if (ACTIVE_RELAY_STATUSES.has(relayStatus)) status = 'working';
  else if (ATTENTION_RELAY_STATUSES.has(relayStatus)) status = 'attention';
  else if (FAILED_RELAY_STATUSES.has(relayStatus)) status = 'degraded';
  else if (DONE_RELAY_STATUSES.has(relayStatus)) status = 'online';
  return { id: 'relay-engine', label: 'Relay Engine', kind: 'relay', status, detail: relayDetail(relayStatus, relayExecution), heartbeat_age_secs: ageFrom(updatedAt, now), session_id: relayExecution.session_id || null, task: null };
}

function buildClaudeTarget(relayExecution, relayStatus, sessionId, taskText, updatedAt, now) {
  if (!sessionId) {
    return { id: 'claude-code', label: 'Claude Code', kind: 'executor', status: 'unknown', detail: '没有活跃执行会话', heartbeat_age_secs: null, session_id: null, task: null };
  }
  let status = 'unknown';
  if (ACTIVE_RELAY_STATUSES.has(relayStatus)) status = 'working';
  else if (DONE_RELAY_STATUSES.has(relayStatus)) status = 'online';
  else if (FAILED_RELAY_STATUSES.has(relayStatus) || ATTENTION_RELAY_STATUSES.has(relayStatus)) status = 'degraded';
  return { id: 'claude-code', label: 'Claude Code', kind: 'executor', status, detail: executorDetail(relayStatus, relayExecution), heartbeat_age_secs: ageFrom(updatedAt, now), session_id: sessionId, task: taskText };
}

function agentVisualState(state, presence) {
  if (state.visual_state_override) return state.visual_state_override;
  const relayExecution = state.relay_execution || state.relayExecution || null;
  const relayStatus = relayExecution && relayExecution.relay_status || '';
  const runningState = state.running_state || state.agentState || 'Stopped';
  const trayState = state.tray_state || '';
  if (state.pending_confirmation || state.pendingConfirmation) return 'confirm';
  if (FAILED_RELAY_STATUSES.has(relayStatus) || ATTENTION_RELAY_STATUSES.has(relayStatus)) return 'error';
  if (DONE_RELAY_STATUSES.has(relayStatus) && relayStatus !== 'interrupted') return 'done';
  if (ACTIVE_RELAY_STATUSES.has(relayStatus)) return 'executing';
  if (trayState === 'processing') return 'analyzing';
  if (runningState === 'Running') return 'monitoring';
  if (runningState === 'Stopped') return 'idle';
  if (presence.overall_status === 'offline') return 'offline';
  return 'idle';
}

function renderMiniWorkstation(state) {
  currentState = state || {};
  const root = document.getElementById('mini-workstation');
  if (!root) return;

  const presence = buildExecutionPresence(currentState);
  const primary = pickPrimaryTarget(presence);
  const action = primaryActionForPresence(currentState);
  const visualState = agentVisualState(currentState, presence);
  const imageSrc = `./assets/agent-states/agent-${visualState}.png`;
  const color = PRESENCE_COLORS[presence.overall_status] || PRESENCE_COLORS.unknown;

  root.style.setProperty('--presence-color', color);
  root.dataset.visualState = visualState;
  root.innerHTML = `
    <div class="mini-main">
      <div class="mini-scene mini-scene-${escapeHtml(visualState)}" aria-hidden="true">
        <img class="mini-agent-img" src="${escapeHtml(imageSrc)}" alt="">
      </div>
      <div class="mini-copy">
        <div class="mini-kicker">WORKSTATION</div>
        <div class="mini-title">${escapeHtml(presenceStatusLabel(presence.overall_status))} · ${escapeHtml(primary && primary.label || '执行端')}</div>
        <div class="mini-summary">${escapeHtml(primary && primary.detail || presence.summary)}</div>
      </div>
    </div>
    <div class="mini-targets">
      ${presence.targets.map(renderTarget).join('')}
    </div>
    <div class="mini-actions">
      <button class="mini-btn ${escapeHtml(action.className)}" data-action="${escapeHtml(action.action)}">${escapeHtml(action.label)}</button>
    </div>
  `;

  root.querySelectorAll('[data-action]').forEach((button) => {
    button.addEventListener('click', () => handleAction(button.dataset.action));
  });
}

function renderTarget(target) {
  const color = PRESENCE_COLORS[target.status] || PRESENCE_COLORS.unknown;
  return `
    <div class="mini-target" title="${escapeHtml(target.label)} · ${escapeHtml(target.detail)} · ${escapeHtml(compactAgeLabel(target.heartbeat_age_secs))}">
      <span class="mini-led" style="--target-color:${escapeHtml(color)}"></span>
      <span class="mini-target-name">${escapeHtml(shortLabel(target.label))}</span>
    </div>
  `;
}

async function init() {
  if (!invoke || !listen) {
    startPreviewLoop();
    return;
  }

  await listen('state-update', (event) => {
    renderMiniWorkstation(event.payload || {});
  });

  try {
    const initialState = await invoke('get_ui_state');
    renderMiniWorkstation(initialState || {});
  } catch (err) {
    console.error('[MiniWorkstation] Failed to get initial state:', err);
    renderMiniWorkstation(currentState);
  }
}

function startPreviewLoop() {
  const now = Math.floor(Date.now() / 1000);
  const states = [
    { visual_state_override: 'idle', running_state: 'Running' },
    { visual_state_override: 'monitoring', running_state: 'Running' },
    { visual_state_override: 'analyzing', tray_state: 'processing', running_state: 'Running' },
    { visual_state_override: 'confirm', running_state: 'Running', pending_confirmation: { task_text: '等待确认', source_window: 'Preview' } },
    { visual_state_override: 'executing', running_state: 'Running', relay_execution: { relay_status: 'running', session_id: 'preview-running', updated_at: now }, current_task: { task_text: '执行预览任务' } },
    { visual_state_override: 'done', running_state: 'Running', relay_execution: { relay_status: 'completed', session_id: 'preview-done', updated_at: now, result_summary: '预览完成' } },
    { visual_state_override: 'error', running_state: 'Running', relay_execution: { relay_status: 'failed', session_id: 'preview-error', updated_at: now, error_message: '预览异常' } },
    { visual_state_override: 'offline', running_state: 'Stopped', relay_execution: { relay_status: 'interrupted', session_id: 'preview-offline', updated_at: now } }
  ];
  let index = 0;
  renderMiniWorkstation(states[index]);
  window.setInterval(() => {
    index = (index + 1) % states.length;
    renderMiniWorkstation(states[index]);
  }, 1600);
}

function primaryActionForPresence(state) {
  if (state.pending_confirmation || state.pendingConfirmation) return { action: 'confirm', label: '确认', className: 'btn-warning' };
  const relayStatus = state.relay_execution && state.relay_execution.relay_status || state.relayExecution && state.relayExecution.relay_status || '';
  if (ACTIVE_RELAY_STATUSES.has(relayStatus)) return { action: 'interrupt', label: '中断', className: 'btn-danger' };
  const runningState = state.running_state || state.agentState || 'Stopped';
  if (runningState === 'Running') return { action: 'toggle', label: '暂停', className: 'btn-secondary' };
  return { action: 'toggle', label: '启动', className: 'btn-primary' };
}

function pickPrimaryTarget(presence) {
  return (presence.targets || []).slice().sort((a, b) => STATUS_PRIORITY[b.status] - STATUS_PRIORITY[a.status])[0] || null;
}

function presenceStatusLabel(status) {
  return PRESENCE_LABELS[status] || PRESENCE_LABELS.unknown;
}

function summaryFor(overallStatus, targets, pendingConfirmation, relayExecution) {
  if (pendingConfirmation) return '有任务正在等待你确认。';
  if (overallStatus === 'degraded') return '至少一个执行端出现异常，需要查看详情。';
  if (overallStatus === 'offline') return '观察循环已停止，执行链路没有主动工作。';
  if (overallStatus === 'attention') return '接力层需要处理，建议查看当前交接。';
  if (overallStatus === 'working') {
    const active = targets.find((target) => target.status === 'working');
    return active ? `${active.label} 正在工作。` : '执行链路正在工作。';
  }
  if (relayExecution && relayExecution.session_id) return '最近的执行会话已收束。';
  return '工位待机，等待观察或派发。';
}

function relayDetail(relayStatus, relayExecution) {
  if (!relayStatus) return '接力层待机';
  if (relayStatus === 'connecting') return '正在连接接力层';
  if (relayStatus === 'dispatching') return '正在派发任务';
  if (relayStatus === 'running') return '会话正在运行';
  if (relayStatus === 'waiting') return '等待执行端响应';
  if (relayStatus === 'interrupting') return '正在中断会话';
  if (relayStatus === 'completed' || relayStatus === 'done') return '最近会话已完成';
  if (relayStatus === 'interrupted') return '最近会话已中断';
  if (relayStatus === 'relay_unavailable') return 'Relay 暂不可用';
  if (relayStatus === 'subscription_error') return '进展订阅失败';
  if (relayStatus === 'dispatch_error') return '派发失败';
  if (relayStatus === 'failed' || relayStatus === 'error') return relayExecution && relayExecution.error_message || 'Relay 报错';
  return relayStatus;
}

function executorDetail(relayStatus, relayExecution) {
  if (ACTIVE_RELAY_STATUSES.has(relayStatus)) return '执行会话活跃';
  if (DONE_RELAY_STATUSES.has(relayStatus)) return '最近任务已结束';
  if (FAILED_RELAY_STATUSES.has(relayStatus) || ATTENTION_RELAY_STATUSES.has(relayStatus)) return relayExecution && relayExecution.error_message || '执行链路异常';
  return '执行端状态未知';
}

function taskTextFrom(currentTask, pendingConfirmation) {
  return currentTask && (currentTask.task_text || currentTask.taskText) || pendingConfirmation && (pendingConfirmation.task_text || pendingConfirmation.taskText) || null;
}

function ageFrom(timestamp, now) {
  if (!timestamp) return null;
  return Math.max(0, now - Number(timestamp));
}

function compactAgeLabel(age) {
  if (age == null) return '无心跳';
  if (age < 3) return '刚刚';
  if (age < 60) return `${age}s 前`;
  return `${Math.floor(age / 60)}m 前`;
}

function shortLabel(label) {
  if (label === '桌面宿主') return 'Host';
  if (label === '观察循环') return 'Loop';
  if (label === 'Relay Engine') return 'Relay';
  if (label === 'Claude Code') return 'Code';
  return label || '-';
}

function escapeHtml(text) {
  if (text == null) return '';
  const div = document.createElement('div');
  div.textContent = String(text);
  return div.innerHTML;
}

init().catch(console.error);
