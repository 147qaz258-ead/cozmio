// cozmio/cozmio_verify/src/main.rs
use base64::{engine::general_purpose::STANDARD, Engine};
use cozmio_core::capture_all;
use cozmio_model::{ask_model_sync, parse_intervention_result, InterventionResult};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 参数解析
    let model = args
        .iter()
        .position(|a| a == "--model")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("qwen2.5-vision");

    let output_dir = args
        .iter()
        .position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("verification/samples"));

    let monitor_index: u32 = args
        .iter()
        .position(|a| a == "--monitor")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    // 捕获窗口信息
    println!("捕获窗口信息 (monitor {})...", monitor_index);
    let capture = match capture_all(monitor_index) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("捕获失败: {}", e);
            std::process::exit(1);
        }
    };

    // 生成样本 ID
    let sample_id = format!(
        "sample_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let sample_dir = output_dir.join(&sample_id);
    fs::create_dir_all(&sample_dir).expect("创建样本目录失败");

    // 保存元信息（只允许机械元信息）
    let meta = serde_json::json!({
        "hwnd": capture.foreground_window.as_ref().map(|w| w.hwnd).unwrap_or(0),
        "title": capture.foreground_window.as_ref().map(|w| w.title.clone()).unwrap_or_default(),
        "process_name": capture.foreground_window.as_ref().map(|w| w.process_name.clone()).unwrap_or_default(),
        "rect": capture.foreground_window.as_ref().map(|w| w.rect.clone()),
        "is_visible": capture.foreground_window.as_ref().map(|w| w.is_visible).unwrap_or(false),
        "is_foreground": capture.foreground_window.as_ref().map(|w| w.is_foreground).unwrap_or(false)
    });

    let meta_path = sample_dir.join("meta.json");
    fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap())
        .expect("写入 meta.json 失败");
    println!("保存元信息: {:?}", meta_path);

    // 保存截图
    if let Some(screenshot) = &capture.screenshot {
        let screenshot_path = sample_dir.join("screenshot.png");
        let image_data = STANDARD
            .decode(&screenshot.image_base64)
            .expect("解码截图失败");
        fs::write(&screenshot_path, &image_data).expect("写入截图失败");
        println!("保存截图: {:?}", screenshot_path);

        // 调用模型（prompt 不引导输出方向）
        println!("调用模型 {}...", model);
        let title = capture
            .foreground_window
            .as_ref()
            .map(|w| w.title.as_str())
            .unwrap_or("");
        let process_name = capture
            .foreground_window
            .as_ref()
            .map(|w| w.process_name.as_str())
            .unwrap_or("");

        let raw_output = ask_model_sync(model, &screenshot.image_base64, title, process_name, "")
            .map_err(|e| {
                eprintln!("模型调用失败: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        let result: InterventionResult = parse_intervention_result(&raw_output)
            .map_err(|e| {
                eprintln!("解析输出失败: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        println!("\n模型判断: {:?}", result.mode);
        println!("理由: {}", result.reason);

        // 保存结构化结果
        let output_json = serde_json::json!({
            "mode": result.mode.to_string(),
            "reason": result.reason,
            "raw_output": raw_output
        });
        let output_path = sample_dir.join("output.json");
        fs::write(
            &output_path,
            serde_json::to_string_pretty(&output_json).unwrap(),
        )
        .expect("写入 output.json 失败");
        println!("保存结构化结果: {:?}", output_path);
    } else {
        eprintln!("未捕获到截图");
        std::process::exit(1);
    }

    println!("\n样本已保存到: {:?}", sample_dir);
    println!("Review 时请对照:");
    println!("  - screenshot.png: 原始截图");
    println!("  - meta.json: 机械元信息");
    println!("  - output.json: 结构化结果 (mode, reason, raw_output)");
}
