use crate::db::Database;
use crate::decision_memory::{Decision, DecisionMemoryStore};
use crate::error::MemoryError;
use crate::skill_memory::{Skill, SkillMemoryStore};
use crate::task_threads::{TaskThreadUpdate, TaskThreadsStore};

/// Decision memory type tags (stored as strings in the DB)
#[derive(Debug, Clone)]
pub enum MemoryType {
    RejectedDirection,
    AcceptedDecision,
    UserPreference,
}

impl MemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryType::RejectedDirection => "RejectedDirection",
            MemoryType::AcceptedDecision => "AcceptedDecision",
            MemoryType::UserPreference => "UserPreference",
        }
    }
}

pub fn seed_demo_data(db: &Database) -> Result<(), MemoryError> {
    // Insert task threads (Section 5.2 in design):
    let threads = TaskThreadsStore::new(db);
    threads.upsert(&TaskThreadUpdate {
        name: "Cozmio 使用体验改造".to_string(),
        current_state: Some("从 CONTINUE/ABSTAIN 弹窗升级为有效建议".to_string()),
        open_questions: Some(vec![
            "如何形成有效建议".to_string(),
            "如何构建最小记忆底座".to_string(),
        ]),
        decisions: Some(vec![
            "端侧负责整理小范围信息".to_string(),
            "执行端处理更大上下文".to_string(),
        ]),
        recent_slice_ids: None,
    })?;

    threads.upsert(&TaskThreadUpdate {
        name: "Cozmio 硬件线 Local Agent Box".to_string(),
        current_state: Some("树莓派 + 端侧部署".to_string()),
        open_questions: Some(vec!["硬件规格".to_string(), "通信方案".to_string()]),
        decisions: None,
        recent_slice_ids: None,
    })?;

    threads.upsert(&TaskThreadUpdate {
        name: "Claude Code 执行链路".to_string(),
        current_state: Some("Relay dispatch + subprocess 管理".to_string()),
        open_questions: None,
        decisions: Some(vec!["Toast → Relay → Claude Code 链路已跑通".to_string()]),
        recent_slice_ids: None,
    })?;

    // Insert decisions (Section 5.2 in design):
    let decisions = DecisionMemoryStore::new(db);
    decisions.insert(&Decision {
        id: None,
        memory_type: MemoryType::RejectedDirection.as_str().to_string(),
        content: "用户反对为了方便绕开向量检索、长期记忆等核心技术".to_string(),
        evidence: None,
        related_thread_id: None,
        evidence_source: "seed".to_string(),
    })?;
    decisions.insert(&Decision {
        id: None,
        memory_type: MemoryType::AcceptedDecision.as_str().to_string(),
        content: "用户不接受牺牲终局体验换取实现便利".to_string(),
        evidence: None,
        related_thread_id: None,
        evidence_source: "seed".to_string(),
    })?;
    decisions.insert(&Decision {
        id: None,
        memory_type: MemoryType::UserPreference.as_str().to_string(),
        content: "用户希望技术方案从好用出发，不从省事出发".to_string(),
        evidence: None,
        related_thread_id: None,
        evidence_source: "seed".to_string(),
    })?;

    // Insert skills (Section 5.2 in design):
    let skills = SkillMemoryStore::new(db);
    skills.insert(&Skill {
        id: None,
        name: "Toast → Relay → Claude Code 执行流程".to_string(),
        description: Some("已跑通的执行链路：Windows Toast 通知 → Relay dispatch → Claude Code subprocess → 结果通知".to_string()),
        procedure: "1. 触发 Toast 通知\n2. 用户点击确认\n3. Relay 接收并解析指令\n4. 启动 Claude Code 子进程\n5. 捕获执行结果\n6. 发送结果通知".to_string(),
        success_context: Some("在 Windows 环境下成功执行了 Cozmio 的 Toast → Relay → Claude Code 完整链路".to_string()),
        usage_count: 1,
        last_used_at: None,
        evidence_source: "seed".to_string(),
    })?;

    Ok(())
}
