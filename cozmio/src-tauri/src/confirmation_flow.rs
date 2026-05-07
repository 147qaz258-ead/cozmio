use crate::relay_bridge::RelayDispatchRequest;
use crate::ui_state::{HandoffPacketInfo, PendingConfirmationInfo};

pub fn relay_request_from_pending_handoff(
    trace_id: &str,
    pending: &PendingConfirmationInfo,
    proposed_task_override: Option<String>,
) -> Result<RelayDispatchRequest, String> {
    let mut packet: HandoffPacketInfo = pending
        .handoff_packet
        .clone()
        .ok_or_else(|| String::from("Pending confirmation missing handoff packet"))?;
    if let Some(task_text) = proposed_task_override {
        if !task_text.trim().is_empty() {
            packet.proposed_task = task_text;
        }
    }
    Ok(RelayDispatchRequest::from_handoff_packet(
        trace_id,
        &pending.task_text,
        &packet,
        &pending.source_window,
        &pending.source_process,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_state::HandoffPacketInfo;

    fn pending_confirmation_with_packet() -> PendingConfirmationInfo {
        PendingConfirmationInfo {
            trace_id: String::from("trace-1"),
            task_text: String::from("我看到你正在复盘 Cozmio 的交接质量。"),
            user_how: None,
            source_window: String::from("Cozmio design review"),
            source_process: String::from("Code.exe"),
            created_at: 1,
            process_context: None,
            context_badges: vec![],
            evidence_cards: vec![],
            validity_age_seconds: Some(0),
            lineage_ref: Some(String::from("lineage-1")),
            handoff_packet: Some(HandoffPacketInfo {
                current_understanding: String::from("用户正在复盘 Cozmio 的交接质量"),
                intervention_reason: String::from("这次现场适合执行端检查实现"),
                executor_target: String::from("claude-code"),
                proposed_task: String::from("检查 handoff packet 是否贯通"),
                task_constraints: vec![String::from("不要扩展到无关重构")],
                evidence_refs: vec![String::from("mem-1")],
            }),
        }
    }

    #[test]
    fn relay_request_from_pending_handoff_requires_packet() {
        let mut pending = pending_confirmation_with_packet();
        pending.handoff_packet = None;

        let err = relay_request_from_pending_handoff("trace-1", &pending, None)
            .expect_err("missing handoff packet should fail");

        assert!(err.contains("missing handoff packet"));
    }

    #[test]
    fn relay_request_from_pending_handoff_uses_packet_not_raw_text() {
        let pending = pending_confirmation_with_packet();

        let request = relay_request_from_pending_handoff(
            "trace-1",
            &pending,
            Some(String::from("执行 Box 生成的具体任务")),
        )
        .expect("handoff packet should produce relay request");

        assert!(request.dispatched_task.contains("执行 Box 生成的具体任务"));
        assert!(request.dispatched_task.contains("mem-1"));
        assert!(request.handoff_packet_json.is_some());
        assert!(!request.dispatched_task.contains("当前理解:"));
        assert!(!request.dispatched_task.contains("请执行的任务:"));
        assert!(!request.dispatched_task.contains("证据引用:"));
    }
}
