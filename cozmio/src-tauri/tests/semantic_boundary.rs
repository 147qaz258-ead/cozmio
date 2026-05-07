use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_runtime_file(relative: &str) -> String {
    let path = crate_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn runtime_region(content: &str) -> String {
    let mut output = String::new();
    let mut skip_cfg_item = false;
    let mut brace_depth: i32 = 0;
    let mut saw_brace = false;

    for line in content.lines() {
        if line.trim() == "#[cfg(test)]" {
            skip_cfg_item = true;
            brace_depth = 0;
            saw_brace = false;
            continue;
        }

        if skip_cfg_item {
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_depth += 1;
                        saw_brace = true;
                    }
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }
            if (!saw_brace && line.trim_end().ends_with(';')) || (saw_brace && brace_depth <= 0) {
                skip_cfg_item = false;
            }
            continue;
        }

        output.push_str(line);
        output.push('\n');
    }

    output
}

#[test]
fn runtime_prompt_does_not_hardcode_model_silence_or_popup_policy() {
    let files = [
        "src/model_client.rs",
        "src/prompt_context.rs",
        "src/main_loop.rs",
        "src/window_monitor.rs",
        "src/commands.rs",
        "src/executor.rs",
        "src/relay_bridge.rs",
        "src/ui_state.rs",
        "src/logging.rs",
        "src/components/StatusPanel.js",
        "src/components/MemoryInspector.js",
        "../cozmio_memory/src/importer.rs",
        "../cozmio_memory/src/slice_builder.rs",
        "../cozmio_memory/src/thread_linker.rs",
    ];
    let forbidden = [
        "保持沉默",
        "弹窗策略",
        "检索线索",
        "project_phase",
        "iteration_opportunity",
        "should_popup",
        "should_silence",
        "popup|silence",
        "popup | silence",
        "frequency cap",
        "cooldown",
        "confidence: 1.0",
        "confidence = 1.0",
        "confidence=1.0",
        "level:",
        "next_step:",
        "系统判断",
        "fake_confidence",
        "is_oscillating",
        "just_arrived",
        "last_switch_direction",
        "mode: continue",
        "mode: abstain",
        "negative_feedback",
        "groundedness_score",
    ];

    for file in files {
        let content = read_runtime_file(file);
        let runtime_content = runtime_region(&content);
        for term in forbidden {
            assert!(
                !runtime_content.to_ascii_lowercase().contains(term),
                "{file} contains forbidden system semantic or popup-control term: {term}"
            );
        }
    }
}

#[test]
fn runtime_prompt_names_system_material_as_facts_not_conclusions() {
    let content = read_runtime_file("src/model_client.rs");

    assert!(content.contains("Cozmio 只提供事实材料和工具材料，不提供结论"));
    assert!(content.contains("不要把它们当成用户意图、任务阶段或项目结论"));
    assert!(content.contains("不要为了迎合上下文而编造屏幕上或材料中没有出现的内容"));
}
