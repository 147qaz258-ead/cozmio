// ============================================================
// Context Admission — 上下文入场管道
//
// "候选池 → Evidence Card → Model Input Packet" 简单通道
//
// 设计原则：
// - 候选池可以大、可以噪、可以包含多条候选
// - 模型输入必须小（固定预算，最多 3 张 Evidence Card）
// - 历史不能原文进入，必须变成短卡片
// - 不用硬阈值，用固定预算
// - 没有合适历史也没关系，当前窗口照样弹
//
// 不负责：
// - 决定是否弹窗（只负责决定哪些历史进入模型输入）
// - 生成语义解释（why_maybe_relevant 只能来自已有摘要/标签/窗口标题）
// - 替模型判断相关性
// ============================================================

use crate::memory_commands::CompetitionResultEntryDto;
use crate::types::{ContextAdmissionLineage, EvidenceCard};

pub const MAX_EVIDENCE_CARDS: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct ContextAdmissionResult {
    pub evidence_cards: Vec<EvidenceCard>,
    pub lineage: ContextAdmissionLineage,
}

pub struct ContextAdmission;

impl ContextAdmission {
    pub fn admit(competition_entries: &[CompetitionResultEntryDto]) -> ContextAdmissionResult {
        let mut candidates: Vec<CandidateEntry<'_>> = competition_entries
            .iter()
            .enumerate()
            .map(|(index, entry)| CandidateEntry {
                entry,
                original_index: index,
                soft_score: compute_soft_score(entry, index),
            })
            .collect();

        candidates.sort_by(|left, right| {
            right
                .soft_score
                .partial_cmp(&left.soft_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.original_index.cmp(&right.original_index))
        });

        let evidence_cards: Vec<EvidenceCard> = candidates
            .into_iter()
            .take(MAX_EVIDENCE_CARDS)
            .map(|candidate| entry_to_card(candidate.entry))
            .collect();

        let selected_card_refs = evidence_cards
            .iter()
            .map(|card| card.ref_id.clone())
            .collect::<Vec<_>>();
        let not_selected_reason = if competition_entries.len() > evidence_cards.len() {
            String::from("budget_only")
        } else {
            String::from("none")
        };

        let lineage = ContextAdmissionLineage {
            candidate_pool_size: competition_entries.len(),
            evidence_cards_selected: evidence_cards.len(),
            selected_card_refs,
            not_selected_reason,
            model_input_packet_summary: format!(
                "current_observation=1, history_evidence_cards={}",
                evidence_cards.len()
            ),
        };

        ContextAdmissionResult {
            evidence_cards,
            lineage,
        }
    }
}

struct CandidateEntry<'a> {
    entry: &'a CompetitionResultEntryDto,
    original_index: usize,
    soft_score: f32,
}

fn compute_soft_score(entry: &CompetitionResultEntryDto, original_index: usize) -> f32 {
    let vector_component = entry.vector_score.unwrap_or(0.0);
    let source_component = if entry.source_event_ids.is_empty() {
        0.0
    } else {
        0.05
    };
    let stable_tiebreaker = 1.0 / ((original_index + 1) as f32 * 10_000.0);

    vector_component + source_component + stable_tiebreaker
}

fn entry_to_card(entry: &CompetitionResultEntryDto) -> EvidenceCard {
    EvidenceCard {
        source: clip(&entry.producer, 80),
        ref_id: clip(&entry.memory_id, 120),
        age_label: None,
        short_summary: clip(&entry.memory_text, 180),
        why_maybe_relevant: entry
            .selection_reason_facts
            .iter()
            .map(|fact| fact.trim())
            .find(|fact| !fact.is_empty())
            .map(|fact| clip(fact, 120)),
        similarity_score: entry.vector_score,
    }
}

fn clip(value: &str, max_chars: usize) -> String {
    let mut clipped: String = value
        .replace('\r', " ")
        .replace('\n', " ")
        .chars()
        .take(max_chars)
        .collect();
    if value.chars().count() > max_chars {
        clipped.push_str("...");
    }
    clipped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, score: Option<f32>, text: &str) -> CompetitionResultEntryDto {
        CompetitionResultEntryDto {
            memory_id: id.to_string(),
            memory_text: text.to_string(),
            memory_kind: String::from("activity"),
            vector_score: score,
            fact_trace: serde_json::json!({ "source": id }),
            selection_reason_facts: vec![format!("source_ref={id}")],
            token_estimate: 42,
            source_event_ids: vec![format!("evt-{id}")],
            source_paths: vec![],
            source_ranges: vec![],
            producer: String::from("memory"),
        }
    }

    #[test]
    fn context_admission_selects_top_three_without_threshold() {
        let entries = vec![
            entry("low", Some(0.1), "low score still enters candidate pool"),
            entry("highest", Some(0.9), "highest score"),
            entry("missing", None, "missing score"),
            entry("middle", Some(0.5), "middle score"),
        ];

        let result = ContextAdmission::admit(&entries);

        assert_eq!(result.lineage.candidate_pool_size, 4);
        assert_eq!(result.evidence_cards.len(), MAX_EVIDENCE_CARDS);
        assert_eq!(
            result.lineage.selected_card_refs,
            vec![
                String::from("highest"),
                String::from("middle"),
                String::from("low")
            ]
        );
        assert_eq!(result.lineage.not_selected_reason, "budget_only");
    }

    #[test]
    fn context_admission_does_not_invent_age_from_token_estimate() {
        let result = ContextAdmission::admit(&[entry("mem-1", Some(0.7), "summary")]);

        assert_eq!(result.evidence_cards[0].age_label, None);
        assert_eq!(
            result.evidence_cards[0].why_maybe_relevant.as_deref(),
            Some("source_ref=mem-1")
        );
    }
}
