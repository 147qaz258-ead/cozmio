use std::fs;

use crate::context_files::{human_context_path, HUMAN_CONTEXT_SOURCE_PATH};
use crate::memory_commands::ReminderContextDto;
use crate::window_monitor::WindowSnapshot;

const MAX_PROMPT_LINE_CHARS: usize = 240;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanMemoryObservation {
    pub existing_human_context: String,
    pub observation_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanMemoryWriteRequest {
    pub signature: String,
    pub prompt: String,
}

pub fn load_human_context() -> String {
    let Some(path) = human_context_path() else {
        return String::new();
    };
    fs::read_to_string(path).unwrap_or_default()
}

pub fn write_human_context(content: &str) -> Result<(), String> {
    let Some(path) = human_context_path() else {
        return Err(String::from("Failed to resolve human_context.md path"));
    };
    fs::write(path, content.trim()).map_err(|err| format!("Failed to write human context: {err}"))
}

pub fn build_observation(
    snapshot: &WindowSnapshot,
    reminder_context: Option<&ReminderContextDto>,
    recent_window_summary: Option<String>,
    recent_executor_summary: Option<String>,
    existing_human_context: String,
) -> HumanMemoryObservation {
    let mut observation_lines = vec![format!(
        "你刚刚看到的前台窗口是《{}》，进程是 {}。",
        clip(&snapshot.window_info.title, MAX_PROMPT_LINE_CHARS),
        clip(&snapshot.window_info.process_name, 80)
    )];

    if let Some(summary) = recent_window_summary
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        observation_lines.push(format!(
            "最近窗口变化里，{}。",
            clip(summary, MAX_PROMPT_LINE_CHARS)
        ));
    }

    if let Some(ctx) = reminder_context {
        if let Some(activity) = non_empty_text(&ctx.current_activity) {
            observation_lines.push(format!(
                "观察循环刚整理出的当前活动是：{}。",
                clip(activity, MAX_PROMPT_LINE_CHARS)
            ));
        }
        if let Some(context) = non_empty_text(&ctx.recent_context) {
            observation_lines.push(format!(
                "近端语境里提到：{}。",
                clip(context, MAX_PROMPT_LINE_CHARS)
            ));
        }
        if let Some(decisions) = non_empty_text(&ctx.related_decisions) {
            observation_lines.push(format!(
                "最近反复出现的决定或纠偏包括：{}。",
                clip(decisions, MAX_PROMPT_LINE_CHARS)
            ));
        }
    }

    if let Some(summary) = non_empty_text(recent_executor_summary.as_deref().unwrap_or("")) {
        observation_lines.push(format!("执行端近期活动：{}", clip(summary, 480)));
    }

    HumanMemoryObservation {
        existing_human_context,
        observation_lines,
    }
}

pub fn build_write_request(
    observation: &HumanMemoryObservation,
    last_signature: Option<&str>,
) -> Option<HumanMemoryWriteRequest> {
    let signature = observation_signature(observation);
    if signature.is_empty() {
        return None;
    }
    if last_signature.is_some_and(|previous| previous == signature) {
        return None;
    }

    Some(HumanMemoryWriteRequest {
        signature,
        prompt: build_memory_update_prompt(observation),
    })
}

pub fn sanitize_model_memory_text(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let without_fence = trimmed
        .strip_prefix("```text")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim();
    without_fence
        .strip_suffix("```")
        .unwrap_or(without_fence)
        .trim()
        .to_string()
}

pub fn build_memory_update_prompt(observation: &HumanMemoryObservation) -> String {
    let existing = if observation.existing_human_context.trim().is_empty() {
        "（当前为空）"
    } else {
        observation.existing_human_context.trim()
    };
    let observed_facts = if observation.observation_lines.is_empty() {
        "这一次没有拿到新的观察事实。".to_string()
    } else {
        observation.observation_lines.join("\n")
    };

    format!(
        "你仍然是 Cozmio 的窗口助手。\n\
\n这一轮除了判断是否该介入，还请顺手维护 `{source_path}` 这个人的电脑记忆文件。\n\
这个文件记的是人，不是项目文档。它的作用是让下一轮观察更快知道：这个人最近在电脑上做什么、反复纠结什么、长期想要什么、刚刚从什么切到了什么。\n\
\n当前记忆文件内容：\n{existing}\n\
\n这一轮你刚刚观察到的事实：\n{observed_facts}\n\
\n请直接输出更新后的记忆文件全文，用自然中文纯文本写。\n\
由你决定保留、改写、追加或压缩哪些内容；如果这轮没有新增价值，也可以基本保持原样。\n\
记忆文件可以包含近期活动、反复纠结的问题、长期意图、刚发生的切换、执行端刚完成或失败的事。\n\
不要输出解释、标题、字段清单、JSON、Markdown 代码块或调试说明，只输出文件内容本身。",
        source_path = HUMAN_CONTEXT_SOURCE_PATH,
        existing = existing,
        observed_facts = observed_facts,
    )
}

fn observation_signature(observation: &HumanMemoryObservation) -> String {
    observation.observation_lines.join(" | ")
}

fn non_empty_text(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn clip(value: &str, max_chars: usize) -> String {
    let mut clipped: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        clipped.push_str("...");
    }
    clipped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_commands::ReminderContextDto;
    use crate::window_monitor::WindowSnapshot;
    use cozmio_core::{Rect, WindowInfo};

    fn snapshot() -> WindowSnapshot {
        WindowSnapshot {
            screenshot_base64: String::new(),
            screenshot_width: 100,
            screenshot_height: 100,
            window_info: WindowInfo {
                hwnd: 0,
                title: String::from("Claude - Cozmio memory design"),
                process_name: String::from("chrome.exe"),
                process_id: 1,
                monitor_index: 0,
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                is_foreground: true,
                is_visible: true,
                z_order: 0,
            },
            timestamp: 1,
        }
    }

    fn reminder() -> ReminderContextDto {
        ReminderContextDto {
            current_activity: String::from("正在讨论 Cozmio 的人的电脑记忆"),
            recent_context: String::from("刚刚讨论了观察循环里顺带写入"),
            related_decisions: String::from("先给写入基础设施，不提前限制模型写法"),
            relevant_skills: String::new(),
            task_state: None,
            evidence_refs: vec![],
            competition_entries: vec![],
            competition_trace: None,
        }
    }

    #[test]
    fn build_write_request_returns_none_for_empty_observation() {
        let obs = HumanMemoryObservation {
            existing_human_context: String::new(),
            observation_lines: vec![],
        };
        assert!(build_write_request(&obs, None).is_none());
    }

    #[test]
    fn build_write_request_skips_same_signature() {
        let obs = build_observation(&snapshot(), Some(&reminder()), None, None, String::new());
        let first = build_write_request(&obs, None).expect("should build first request");
        assert!(build_write_request(&obs, Some(&first.signature)).is_none());
    }

    #[test]
    fn build_memory_update_prompt_supports_blank_memory() {
        let obs = build_observation(
            &snapshot(),
            Some(&reminder()),
            Some(String::from("最近窗口切换到浏览器")),
            None,
            String::new(),
        );
        let prompt = build_memory_update_prompt(&obs);
        assert!(prompt.contains("你仍然是 Cozmio 的窗口助手"));
        assert!(prompt.contains("（当前为空）"));
        assert!(prompt.contains("只输出文件内容本身"));
        assert!(prompt.contains("由你决定保留、改写、追加或压缩哪些内容"));
        assert!(!prompt.contains("你可以保持很短"));
        assert!(!prompt.contains("窗口标题："));
    }

    #[test]
    fn build_observation_includes_recent_executor_summary() {
        let obs = build_observation(
            &snapshot(),
            Some(&reminder()),
            None,
            Some(String::from("Relay session completed: 修复观察链路")),
            String::new(),
        );

        assert!(obs
            .observation_lines
            .iter()
            .any(|line| line.contains("执行端近期活动：Relay session completed")));
    }

    #[test]
    fn sanitize_model_memory_text_removes_fences() {
        let value = sanitize_model_memory_text("```text\n今天在做：测试\n```");
        assert_eq!(value, "今天在做：测试");
    }
}
