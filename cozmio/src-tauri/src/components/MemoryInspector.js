const { invoke } = window.__TAURI__.core;

export function createMemoryInspector() {
    const panel = document.createElement('div');
    panel.id = 'panel-memory';
    panel.className = 'content-panel';
    panel.innerHTML = `
        <div class="panel-title">MEMORY</div>
        <div class="btn-group memory-toolbar">
            <button class="btn btn-primary" data-action="refresh-memory">Refresh</button>
            <button class="btn btn-secondary" data-action="build-packet">Build Packet</button>
            <button class="btn btn-secondary" data-action="view-hot-context">Hot Context</button>
            <button class="btn btn-secondary" data-action="run-consolidation">Run Consolidation</button>
            <button class="btn btn-secondary" data-action="replay-selected" style="display:none">Replay Selected</button>
        </div>
        <div id="memory-build-output" class="memory-build-output"></div>
        <div id="memory-inspector-body" class="memory-inspector-body">
            <div class="empty-state">No memory snapshot loaded</div>
        </div>
    `;

    // Track selected source refs
    const selectedSources = new Set();

    const updateReplayButton = () => {
        const btn = panel.querySelector('[data-action="replay-selected"]');
        if (btn) {
            btn.style.display = selectedSources.size >= 1 ? 'inline-block' : 'none';
        }
    };

    panel.addEventListener('click', async (event) => {
        const button = event.target.closest('button[data-action]');
        if (!button) return;
        const action = button.dataset.action;
        if (action === 'refresh-memory') {
            await loadMemoryInspector(panel);
            return;
        }
        if (action === 'build-packet') {
            await buildPacketPreview(panel);
            return;
        }
        if (action === 'reject-memory') {
            const memoryId = button.dataset.memoryId;
            const reason = window.prompt('Record rejection text for this memory row:', '');
            await invoke('reject_agent_memory', {
                memoryId,
                reason: reason || null,
            });
            await loadMemoryInspector(panel);
        }
        if (action === 'supersede-memory') {
            const memoryId = button.dataset.memoryId;
            const newBody = window.prompt('Enter new body text to supersede this memory:', '');
            if (newBody === null) return;
            await invoke('apply_memory_operation', {
                packet: { packet_id: '', created_at: 0, sources: [], related_memories: [], current_hot_context: '' },
                input: {
                    operation_type: 'remove_or_supersede',
                    target_memory_id: memoryId,
                    body: newBody,
                    layer: null,
                    source_refs: [],
                    producer: 'user',
                },
            });
            await loadMemoryInspector(panel);
        }
        if (action === 'apply-proposal') {
            const memoryId = button.dataset.memoryId;
            const memories = button.dataset.memories ? JSON.parse(button.dataset.memories) : [];
            const memory = memories.find(m => m.memory_id === memoryId);
            if (!memory) {
                alert('Memory not found');
                return;
            }
            await invoke('set_hot_context', { content: memory.body });
            alert('Hot context (human_context.md) updated from proposal.');
        }
        if (action === 'view-hot-context') {
            await viewHotContext(panel);
        }
        if (action === 'toggle-source-select') {
            const sourceRef = button.dataset.sourceRef;
            if (selectedSources.has(sourceRef)) {
                selectedSources.delete(sourceRef);
                button.classList.remove('btn-selected');
            } else {
                selectedSources.add(sourceRef);
                button.classList.add('btn-selected');
            }
            updateReplayButton();
        }
        if (action === 'run-consolidation') {
            const output = panel.querySelector('#memory-build-output');
            if (output) output.textContent = 'Running consolidation...';
            try {
                await invoke('run_manual_memory_consolidation', {
                    triggerKind: 'manual_dev',
                    operations: [{ operation_type: 'abstain', layer: null, body: null, source_refs: [], target_memory_id: null, producer: 'user' }],
                });
                if (output) output.innerHTML = '<div class="empty-state">Consolidation run completed.</div>';
                await loadMemoryInspector(panel);
            } catch (err) {
                if (output) output.innerHTML = `<div class="empty-state">Consolidation failed: ${escapeHtml(String(err))}</div>`;
            }
        }
        if (action === 'replay-selected') {
            if (selectedSources.size === 0) return;
            const output = panel.querySelector('#memory-build-output');
            if (output) output.textContent = 'Building replay comparison report...';
            try {
                const result = await invoke('build_replay_comparison_report', { sourceRefs: Array.from(selectedSources) });
                if (output) {
                    output.innerHTML = renderComparisonReport(result);
                }
            } catch (err) {
                if (output) output.innerHTML = `<div class="empty-state">Replay failed: ${escapeHtml(String(err))}</div>`;
            }
        }
    });

    return panel;
}

export async function loadMemoryInspector(panel) {
    const body = panel.querySelector('#memory-inspector-body');
    if (!body) return;
    body.innerHTML = '<div class="empty-state">Loading memory snapshot...</div>';
    try {
        const snapshot = await invoke('get_memory_inspector_snapshot', { limit: 40 });
        const privacyStatus = await invoke('get_privacy_routing_status').catch(() => null);
        const procedureStats = await invoke('get_procedure_recall_stats').catch(() => null);
        body.innerHTML = renderSnapshot(snapshot || {}, privacyStatus, procedureStats);
    } catch (err) {
        body.innerHTML = `<div class="empty-state">Failed to load memory snapshot: ${escapeHtml(String(err))}</div>`;
    }
}

async function buildPacketPreview(panel) {
    const output = panel.querySelector('#memory-build-output');
    if (!output) return;
    output.textContent = 'Building packet...';
    try {
        const preview = await invoke('build_memory_consolidation_prompt');
        output.innerHTML = `
            <div class="module-card memory-preview">
                <div class="memory-row-title">packet_id=${escapeHtml(preview.packet_id || '-')}</div>
                <pre class="memory-pre">${escapeHtml(preview.prompt || '')}</pre>
            </div>
        `;
    } catch (err) {
        output.innerHTML = `<div class="empty-state">Failed to build packet: ${escapeHtml(String(err))}</div>`;
    }
}

async function viewHotContext(panel) {
    const body = panel.querySelector('#memory-inspector-body');
    if (!body) return;
    try {
        const content = await invoke('get_hot_context');
        const editPrompt = window.prompt('Hot Context (human_context.md) content. Edit to update:', content);
        if (editPrompt !== null && editPrompt !== content) {
            await invoke('set_hot_context', { content: editPrompt });
        }
        // Show current content
        body.innerHTML = renderSnapshot({});
        const output = panel.querySelector('#memory-build-output');
        if (output) {
            output.innerHTML = `
                <div class="module-card memory-preview">
                    <div class="memory-row-title">human_context.md</div>
                    <pre class="memory-pre">${escapeHtml(content || '(empty)')}</pre>
                </div>
            `;
        }
    } catch (err) {
        body.innerHTML = `<div class="empty-state">Failed to load hot context: ${escapeHtml(String(err))}</div>`;
    }
}

function renderSnapshot(snapshot, privacyStatus, procedureStats) {
    return `
        ${renderPrivacyStatus(privacyStatus)}
        ${renderProcedureStats(procedureStats)}
        ${renderRecallAdmission(snapshot.latest_runtime_recall_admission)}
        ${renderMemories(snapshot.memories || [])}
        ${renderSources(snapshot.recent_experience_sources || [])}
        ${renderRuns(snapshot.recent_consolidation_runs || [])}
        ${renderOperations(snapshot.recent_operations || [])}
    `;
}

function renderPrivacyStatus(status) {
    if (!status) {
        return section('PRIVACY ROUTING', '<div class="empty-state">Privacy status unavailable</div>');
    }
    const routeInfo = status.route_distribution && status.route_distribution.length > 0
        ? status.route_distribution.map(r => `${r.route}: ${r.count}`).join(', ')
        : 'no runs yet';
    return section('PRIVACY ROUTING', `
        <div class="memory-row">
            <div class="memory-row-head">
                <span class="memory-row-title">mode=${escapeHtml(status.current_mode || 'unknown')}</span>
            </div>
            <div class="memory-meta">allowed_material_count=${status.allowed_material_ids ? status.allowed_material_ids.length : 0}</div>
            <div class="memory-meta">route_distribution=${escapeHtml(routeInfo)}</div>
            <div class="memory-meta">distinct_routes=${status.consolidation_routes ? status.consolidation_routes.length : 0}</div>
            ${status.last_approval_source_refs && status.last_approval_source_refs.length > 0
                ? `<div class="memory-meta">last_approval_sources=${escapeHtml(status.last_approval_source_refs.join(', '))}</div>`
                : ''}
        </div>
    `);
}

function renderProcedureStats(stats) {
    if (!stats) {
        return section('PROCEDURE RECALL', '<div class="empty-state">Procedure stats unavailable</div>');
    }
    const topProcedures = stats.top_procedures && stats.top_procedures.length > 0
        ? stats.top_procedures.map(p => `
            <div class="memory-row">
                <div class="memory-row-head">
                    <span class="memory-row-title">#${escapeHtml(p.memory_id)}</span>
                    <span class="memory-meta">use_count=${p.use_count} last_used=${formatTime(p.last_used_at)}</span>
                </div>
                <div class="memory-body">${escapeHtml(p.body_preview || '')}</div>
            </div>`).join('')
        : '<div class="empty-state">No procedures recorded</div>';
    return section('PROCEDURE RECALL', `
        <div class="memory-row">
            <div class="memory-row-head">
                <span class="memory-row-title">total=${stats.total_procedures} active=${stats.active_procedures}</span>
                <span class="memory-meta">total_use_count=${stats.total_use_count}</span>
            </div>
        </div>
        ${topProcedures}
    `);
}

function renderRecallAdmission(admission) {
    if (!admission) {
        return section('LATEST RECALL ADMISSION', '<div class="empty-state">No runtime recall admission recorded</div>');
    }
    const b = admission.budget || {};
    const hc = admission.hot_context;
    const facts = admission.feedback_facts || [];
    const mems = admission.memories || [];
    return section('LATEST RECALL ADMISSION', `
        <div class="memory-row">
            <div class="memory-row-head">
                <span class="memory-row-title">captured_at=${formatTime(admission.captured_at)} route=${escapeHtml(admission.route)}</span>
            </div>
            <div class="memory-meta">context_surface=${escapeHtml(admission.context_surface || '-')} window=${escapeHtml(admission.window_title || '-')} process=${escapeHtml(admission.process_name || '-')}</div>
            <div class="memory-meta">budget: max_memories=${b.max_memories} max_feedback_facts=${b.max_recent_feedback_facts} max_hot_context_chars=${b.max_hot_context_chars} max_memory_chars=${b.max_memory_chars}</div>
        </div>
        ${hc ? `<div class="memory-row">
            <div class="memory-row-head"><span class="memory-row-title">hot_context admitted</span></div>
            <div class="memory-meta">source_type=${escapeHtml(hc.source_type)} chars=${hc.chars} admitted_chars=${hc.admitted_chars}</div>
            <div class="memory-meta">reason=${escapeHtml(hc.mechanical_reason)}</div>
        </div>` : ''}
        ${facts.length ? `<div class="memory-row-head" style="margin-top:4px"><span class="memory-row-title">feedback_facts (${facts.length})</span></div>
            ${facts.map(f => `
            <div class="memory-row">
                <div class="memory-meta">${escapeHtml(f.event_kind)} age=${f.age_seconds}s score=${f.mechanical_score} reason=${escapeHtml(f.mechanical_reason)}</div>
                <div class="memory-body">${escapeHtml(f.factual_text || '')}</div>
            </div>`).join('')}` : ''}
        ${mems.length ? `<div class="memory-row-head" style="margin-top:4px"><span class="memory-row-title">admitted_memories (${mems.length})</span></div>
            ${mems.map(m => `
            <div class="memory-row">
                <div class="memory-row-head"><span class="memory-row-title">#${escapeHtml(m.memory_id)} ${escapeHtml(m.layer)} score=${m.mechanical_score}</span></div>
                <div class="memory-body">${escapeHtml(m.body || '')}</div>
                <div class="memory-meta">reason=${escapeHtml(m.mechanical_reason)}</div>
            </div>`).join('')}` : ''}
    `);
}

function renderMemories(memories) {
    if (!memories.length) {
        return section('MEMORY ROWS', '<div class="empty-state">No memory rows</div>');
    }
    return section('MEMORY ROWS', memories.map((memory) => `
        <div class="memory-row">
            <div class="memory-row-head">
                <span class="memory-row-title">#${escapeHtml(memory.memory_id)} ${escapeHtml(memory.layer)} / ${escapeHtml(memory.lifecycle)}</span>
                <span class="memory-meta">used=${escapeHtml(memory.use_count)} producer=${escapeHtml(memory.producer || '-')}</span>
            </div>
            <div class="memory-body">${escapeHtml(memory.body || '')}</div>
            <div class="memory-meta">source_refs=${escapeHtml((memory.source_refs || []).join(', ') || '-')}</div>
            <div class="memory-meta">supersedes=${escapeHtml(memory.supersedes || '-')} last_used_at=${formatTime(memory.last_used_at)}</div>
            ${memory.lifecycle === 'active' ? `
                <button class="btn btn-danger btn-small" data-action="reject-memory" data-memory-id="${escapeHtml(memory.memory_id)}">Reject</button>
                <button class="btn btn-secondary btn-small" data-action="supersede-memory" data-memory-id="${escapeHtml(memory.memory_id)}">Supersede</button>
            ` : ''}
            ${memory.layer === 'hot_context_proposal' ? `
                <button class="btn btn-secondary btn-small" data-action="apply-proposal" data-memory-id="${escapeHtml(memory.memory_id)}" data-memories="${escapeHtml(JSON.stringify(memories))}">Apply Proposal</button>
            ` : ''}
        </div>
    `).join(''));
}

function renderSources(sources) {
    if (!sources.length) {
        return section('RECENT FACTS', '<div class="empty-state">No recent experience sources</div>');
    }
    return section('RECENT FACTS', sources.map((source) => `
        <div class="memory-row">
            <div class="memory-row-head">
                <button class="btn btn-small btn-select" data-action="toggle-source-select" data-source-ref="${escapeHtml(source.source_ref)}">[ ]</button>
                <span class="memory-row-title">${escapeHtml(source.source_ref)}</span>
                <span class="memory-meta">${escapeHtml(source.event_kind)} ${escapeHtml(source.timestamp || '')}</span>
            </div>
            <div class="memory-body">${escapeHtml(source.factual_text || '')}</div>
            <div class="memory-meta">trace_id=${escapeHtml(source.trace_id || '-')} raw_ref=${escapeHtml(source.raw_ref || '-')}</div>
        </div>
    `).join(''));
}

function renderRuns(runs) {
    if (!runs.length) {
        return section('CONSOLIDATION RUNS', '<div class="empty-state">No consolidation runs</div>');
    }
    return section('CONSOLIDATION RUNS', runs.map((run) => `
        <div class="memory-row">
            <div class="memory-row-head">
                <span class="memory-row-title">run=${escapeHtml(run.run_id)} ${escapeHtml(run.status)}</span>
                <span class="memory-meta">${formatTime(run.created_at)} route=${escapeHtml(run.route)}</span>
            </div>
            <div class="memory-meta">trigger=${escapeHtml(run.trigger_kind)} packet=${escapeHtml(run.packet_id || '-')} sources=${escapeHtml(run.packet_source_count)} related_memories=${escapeHtml(run.packet_related_memory_count)}</div>
            ${run.error_text ? `<div class="memory-body">${escapeHtml(run.error_text)}</div>` : ''}
        </div>
    `).join(''));
}

function renderOperations(operations) {
    if (!operations.length) {
        return section('MEMORY OPERATIONS', '<div class="empty-state">No memory operations</div>');
    }
    return section('MEMORY OPERATIONS', operations.map((operation) => `
        <div class="memory-row">
            <div class="memory-row-head">
                <span class="memory-row-title">op=${escapeHtml(operation.operation_id)} ${escapeHtml(operation.operation_type)}</span>
                <span class="memory-meta">${formatTime(operation.created_at)} ${escapeHtml(operation.status)}</span>
            </div>
            <div class="memory-meta">target=${escapeHtml(operation.target_memory_id || '-')} result=${escapeHtml(operation.resulting_memory_id || '-')} producer=${escapeHtml(operation.producer || '-')}</div>
            <div class="memory-meta">source_refs=${escapeHtml((operation.source_refs || []).join(', ') || '-')}</div>
            ${operation.body ? `<div class="memory-body">${escapeHtml(operation.body)}</div>` : ''}
        </div>
    `).join(''));
}

function renderComparisonReport(result) {
    return `
    <div class="comparison-report">
        <div class="comparison-header">Replay Comparison Report</div>
        <div class="comparison-section">
            <div class="comparison-label">WITH MEMORY (${result.admitted_memory_count} memories, ${result.hot_context_chars} chars hot context)</div>
            <pre class="memory-pre">${escapeHtml(result.with_memory_preview || '(empty)')}</pre>
        </div>
        <div class="comparison-section">
            <div class="comparison-label">WITHOUT MEMORY (baseline)</div>
            <pre class="memory-pre">${escapeHtml(result.without_memory_preview || '(empty)')}</pre>
        </div>
        <div class="comparison-footer">sources: ${result.source_count}</div>
    </div>
    `;
}

function section(title, content) {
    return `
        <div class="panel-title panel-title-secondary">${title}</div>
        <div class="module-card memory-section">${content}</div>
    `;
}

function formatTime(value) {
    if (!value) return '-';
    const date = new Date(Number(value) * 1000);
    if (Number.isNaN(date.getTime())) return escapeHtml(String(value));
    return escapeHtml(date.toLocaleString());
}

function escapeHtml(value) {
    return String(value ?? '')
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}
