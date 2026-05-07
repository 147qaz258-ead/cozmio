const ACTIVE_RELAY_STATUSES = new Set(['connecting', 'dispatching', 'running', 'waiting', 'interrupting']);
const ATTENTION_RELAY_STATUSES = new Set(['relay_unavailable', 'subscription_error', 'interrupt_error']);
const FAILED_RELAY_STATUSES = new Set(['failed', 'error', 'dispatch_error']);
const DONE_RELAY_STATUSES = new Set(['completed', 'done', 'interrupted']);

export const PRESENCE_LABELS = {
    online: '在线',
    working: '工作中',
    attention: '待处理',
    degraded: '异常',
    offline: '离线',
    unknown: '待机'
};

export const PRESENCE_COLORS = {
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

export function buildExecutionPresence(state = {}) {
    const now = Math.floor(Date.now() / 1000);
    const runningState = state.running_state || state.agentState || 'Stopped';
    const trayState = state.tray_state || '';
    const relayExecution = state.relay_execution || state.relayExecution || null;
    const currentTask = state.current_task || state.currentTask || null;
    const pendingConfirmation = state.pending_confirmation || state.pendingConfirmation || null;
    const relayStatus = relayExecution?.relay_status || '';
    const relayUpdatedAt = relayExecution?.updated_at || null;
    const sessionId = relayExecution?.session_id || null;
    const taskText = taskTextFrom(currentTask, pendingConfirmation);

    const targets = [
        {
            id: 'desktop-host',
            label: '桌面宿主',
            kind: 'host',
            status: 'online',
            detail: '主进程正在上报状态',
            heartbeat_age_secs: 0,
            session_id: null,
            task: null
        },
        {
            id: 'monitor-loop',
            label: '观察循环',
            kind: 'monitor',
            status: trayState === 'processing' ? 'working' : runningState === 'Running' ? 'online' : 'offline',
            detail: trayState === 'processing'
                ? '正在分析前台窗口'
                : runningState === 'Running'
                    ? `以 ${state.poll_interval_secs || state.interval || 3}s 间隔观察`
                    : '监控循环已停止',
            heartbeat_age_secs: null,
            session_id: null,
            task: null
        },
        buildRelayTarget(relayExecution, relayStatus, relayUpdatedAt, now),
        buildClaudeTarget(relayExecution, relayStatus, sessionId, taskText, relayUpdatedAt, now)
    ];

    const overallStatus = computeOverallStatus(targets, pendingConfirmation);
    return {
        overall_status: overallStatus,
        summary: summaryFor(overallStatus, targets, pendingConfirmation, relayExecution),
        targets,
        updated_at: now
    };
}

export function agentVisualState(state = {}, presence = null) {
    if (state.visual_state_override) return state.visual_state_override;
    const computedPresence = presence || buildExecutionPresence(state);
    const relayExecution = state.relay_execution || state.relayExecution || null;
    const relayStatus = relayExecution?.relay_status || '';
    const runningState = state.running_state || state.agentState || 'Stopped';
    const trayState = state.tray_state || '';

    if (state.pending_confirmation || state.pendingConfirmation) return 'confirm';
    if (FAILED_RELAY_STATUSES.has(relayStatus) || ATTENTION_RELAY_STATUSES.has(relayStatus)) return 'error';
    if (DONE_RELAY_STATUSES.has(relayStatus) && relayStatus !== 'interrupted') return 'done';
    if (ACTIVE_RELAY_STATUSES.has(relayStatus)) return 'executing';
    if (trayState === 'processing') return 'analyzing';
    if (runningState === 'Running') return 'monitoring';
    if (computedPresence.overall_status === 'offline') return 'offline';
    return 'idle';
}

export function agentImageForVisualState(visualState) {
    const normalized = visualState || 'idle';
    return `./assets/agent-states/agent-${normalized}.png`;
}

function buildRelayTarget(relayExecution, relayStatus, updatedAt, now) {
    if (!relayExecution) {
        return {
            id: 'relay-engine',
            label: 'Relay Engine',
            kind: 'relay',
            status: 'unknown',
            detail: '尚无接力会话，等待首次派发',
            heartbeat_age_secs: null,
            session_id: null,
            task: null
        };
    }

    let status = 'unknown';
    if (ACTIVE_RELAY_STATUSES.has(relayStatus)) status = 'working';
    else if (ATTENTION_RELAY_STATUSES.has(relayStatus)) status = 'attention';
    else if (FAILED_RELAY_STATUSES.has(relayStatus)) status = 'degraded';
    else if (DONE_RELAY_STATUSES.has(relayStatus)) status = 'online';

    return {
        id: 'relay-engine',
        label: 'Relay Engine',
        kind: 'relay',
        status,
        detail: relayDetail(relayStatus, relayExecution),
        heartbeat_age_secs: ageFrom(updatedAt, now),
        session_id: relayExecution.session_id || null,
        task: null
    };
}

function buildClaudeTarget(relayExecution, relayStatus, sessionId, taskText, updatedAt, now) {
    if (!sessionId) {
        return {
            id: 'claude-code',
            label: 'Claude Code',
            kind: 'executor',
            status: 'unknown',
            detail: '没有活跃执行会话',
            heartbeat_age_secs: null,
            session_id: null,
            task: null
        };
    }

    let status = 'unknown';
    if (ACTIVE_RELAY_STATUSES.has(relayStatus)) status = 'working';
    else if (DONE_RELAY_STATUSES.has(relayStatus)) status = 'online';
    else if (FAILED_RELAY_STATUSES.has(relayStatus) || ATTENTION_RELAY_STATUSES.has(relayStatus)) status = 'degraded';

    return {
        id: 'claude-code',
        label: 'Claude Code',
        kind: 'executor',
        status,
        detail: executorDetail(relayStatus, relayExecution),
        heartbeat_age_secs: ageFrom(updatedAt, now),
        session_id: sessionId,
        task: taskText
    };
}

function computeOverallStatus(targets, pendingConfirmation) {
    if (pendingConfirmation) return 'attention';
    return targets.reduce((highest, target) => {
        return STATUS_PRIORITY[target.status] > STATUS_PRIORITY[highest] ? target.status : highest;
    }, 'unknown');
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
    if (relayExecution?.session_id) return '最近的执行会话已收束。';
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
    if (relayStatus === 'failed' || relayStatus === 'error') return relayExecution?.error_message || 'Relay 报错';
    return relayStatus;
}

function executorDetail(relayStatus, relayExecution) {
    if (ACTIVE_RELAY_STATUSES.has(relayStatus)) return '执行会话活跃';
    if (DONE_RELAY_STATUSES.has(relayStatus)) return '最近任务已结束';
    if (FAILED_RELAY_STATUSES.has(relayStatus) || ATTENTION_RELAY_STATUSES.has(relayStatus)) {
        return relayExecution?.error_message || '执行链路异常';
    }
    return '执行端状态未知';
}

function taskTextFrom(currentTask, pendingConfirmation) {
    return currentTask?.task_text || currentTask?.taskText || pendingConfirmation?.task_text || pendingConfirmation?.taskText || null;
}

function ageFrom(timestamp, now) {
    if (!timestamp) return null;
    return Math.max(0, now - Number(timestamp));
}

export function presenceStatusLabel(status) {
    return PRESENCE_LABELS[status] || PRESENCE_LABELS.unknown;
}

export function presenceToneClass(status) {
    return `tone-${status || 'unknown'}`;
}

export function pickPrimaryTarget(presence) {
    const targets = presence?.targets || [];
    return targets.slice().sort((a, b) => STATUS_PRIORITY[b.status] - STATUS_PRIORITY[a.status])[0] || null;
}

export function primaryActionForPresence(state = {}, presence = {}) {
    if (state.pending_confirmation || state.pendingConfirmation) {
        return { action: 'confirm', label: '确认', className: 'btn-warning' };
    }

    const relayStatus = state.relay_execution?.relay_status || state.relayExecution?.relay_status || '';
    if (ACTIVE_RELAY_STATUSES.has(relayStatus)) {
        return { action: 'interrupt', label: '中断', className: 'btn-danger' };
    }

    const runningState = state.running_state || state.agentState || 'Stopped';
    if (runningState === 'Running') {
        return { action: 'toggle', label: '暂停', className: 'btn-secondary' };
    }

    return { action: 'toggle', label: '启动', className: 'btn-primary' };
}

export function compactAgeLabel(age) {
    if (age == null) return '无心跳';
    if (age < 3) return '刚刚';
    if (age < 60) return `${age}s 前`;
    return `${Math.floor(age / 60)}m 前`;
}
