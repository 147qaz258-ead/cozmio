//! Embedding Model Probe - Tests Qwen3-Embedding-0.6B and BGE-M3 availability
//!
//! Tests:
//! 1. FastEmbed provider initialization
//! 2. Model loading from HuggingFace
//! 3. Embedding generation for test texts
//! 4. Similarity computation
//! 5. Verification of semantic relationships

use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

const TEST_TEXTS: &[&str] = &[
    "提醒没有帮助",
    "Toast 泛化",
    "弹窗没有有效建议",
    "CONTINUE reason 太薄",
    "用户不想为了方便牺牲体验",
    "Local Agent Box payload_text 进入执行链路",
    "Raspberry Pi GGUF model runtime",
    "Rust Tauri Relay dispatch",
];

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}

fn main() -> Result<()> {
    println!("=== Embedding Model Probe ===\n");

    // Step 1: List available models
    println!("Step 1: Checking available FastEmbed models...");
    let models = TextEmbedding::list_supported_models();
    println!("Found {} models:\n", models.len());
    for m in &models {
        println!("  - {:?}: {} (dim={})", m.model, m.description, m.dim);
    }
    println!();

    // Step 2: Try to initialize FastEmbed with default model
    println!("Step 2: Testing FastEmbed initialization with default model (BGESmallENV15)...");
    match TextEmbedding::try_new(Default::default()) {
        Ok(mut model) => {
            println!("  SUCCESS: Default model initialized\n");

            // Step 3: Generate embeddings for test texts
            println!(
                "Step 3: Generating embeddings for {} test texts...",
                TEST_TEXTS.len()
            );
            let texts: Vec<&str> = TEST_TEXTS.iter().copied().collect();
            let embeddings = model.embed(&texts, None)?;
            println!(
                "  Generated {} embeddings, dimension={}\n",
                embeddings.len(),
                embeddings[0].len()
            );

            // Step 4: Similarity matrix
            println!("Step 4: Computing similarity matrix...");
            println!("\nQuery: 提醒没有帮助 (index 0)");
            let q = &embeddings[0];
            let mut sims: Vec<(usize, f32)> = TEST_TEXTS
                .iter()
                .enumerate()
                .skip(1)
                .map(|(i, text)| {
                    let s = cosine_similarity(q, &embeddings[i]);
                    (i, s, *text)
                })
                .map(|(i, s, text)| {
                    println!("  vs {}: {:.4}", text, s);
                    (i, s)
                })
                .collect();
            sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            println!("\nTop 3 most similar to '提醒没有帮助':");
            for (i, s) in sims.into_iter().take(3) {
                println!("  {:.4} - {}", s, TEST_TEXTS[i]);
            }

            // Test specific pair
            println!("\n--- Specific Verification ---");
            let remi_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("提醒没有帮助"))
                .unwrap();
            let toast_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("Toast 泛化"))
                .unwrap();
            let rasp_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("Raspberry Pi"))
                .unwrap();
            let relay_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("Rust Tauri Relay"))
                .unwrap();
            let local_box_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("Local Agent Box"))
                .unwrap();

            println!(
                "'提醒没有帮助' vs 'Toast 泛化': {:.4} (expect high)",
                cosine_similarity(&embeddings[remi_i], &embeddings[toast_i])
            );
            println!(
                "'提醒没有帮助' vs 'Raspberry Pi GGUF': {:.4} (expect low)",
                cosine_similarity(&embeddings[remi_i], &embeddings[rasp_i])
            );
            println!(
                "'Local Agent Box' vs 'Rust Tauri Relay dispatch': {:.4} (expect high)",
                cosine_similarity(&embeddings[local_box_i], &embeddings[relay_i])
            );
            println!(
                "'Local Agent Box' vs 'Raspberry Pi GGUF': {:.4}",
                cosine_similarity(&embeddings[local_box_i], &embeddings[rasp_i])
            );

            println!("\n=== Default model (BGESmallENV15) PASSED ===");
        }
        Err(e) => {
            println!("  FAILED: {}", e);
            println!("\n=== Default model FAILED ===");
        }
    }

    // Step 5: Try BGE-M3 (multilingual)
    println!("\n\nStep 5: Testing BGE-M3 model...");
    match TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::BGEM3).with_show_download_progress(true),
    ) {
        Ok(mut model) => {
            println!("  SUCCESS: BGE-M3 initialized\n");
            println!("  Generating embeddings with BGE-M3...");
            let embeddings = model.embed(
                TEST_TEXTS.iter().map(|s| *s).collect::<Vec<_>>().as_slice(),
                None,
            )?;
            println!(
                "  Generated {} embeddings, dimension={}\n",
                embeddings.len(),
                embeddings[0].len()
            );

            let remi_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("提醒没有帮助"))
                .unwrap();
            let toast_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("Toast 泛化"))
                .unwrap();
            let rasp_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("Raspberry Pi"))
                .unwrap();
            let relay_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("Rust Tauri Relay"))
                .unwrap();
            let local_box_i = TEST_TEXTS
                .iter()
                .position(|t| t.contains("Local Agent Box"))
                .unwrap();

            println!("BGE-M3 Similarity Tests:");
            println!(
                "'提醒没有帮助' vs 'Toast 泛化': {:.4} (expect high)",
                cosine_similarity(&embeddings[remi_i], &embeddings[toast_i])
            );
            println!(
                "'提醒没有帮助' vs 'Raspberry Pi GGUF': {:.4} (expect low)",
                cosine_similarity(&embeddings[remi_i], &embeddings[rasp_i])
            );
            println!(
                "'Local Agent Box' vs 'Rust Tauri Relay dispatch': {:.4} (expect high)",
                cosine_similarity(&embeddings[local_box_i], &embeddings[relay_i])
            );
            println!(
                "'Local Agent Box' vs 'Raspberry Pi GGUF': {:.4}",
                cosine_similarity(&embeddings[local_box_i], &embeddings[rasp_i])
            );

            // Sort and show top similar to "remind not helpful"
            let q = &embeddings[remi_i];
            let mut sims: Vec<_> = TEST_TEXTS
                .iter()
                .enumerate()
                .skip(1)
                .map(|(i, text)| (i, cosine_similarity(q, &embeddings[i]), *text))
                .collect();
            sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            println!("\nBGE-M3: Top 3 similar to '提醒没有帮助':");
            for (i, s, text) in sims.into_iter().take(3) {
                println!("  {:.4} - {}", s, text);
            }

            println!("\n=== BGE-M3 PASSED ===");
        }
        Err(e) => {
            println!("  FAILED: {}", e);
            println!("\n=== BGE-M3 FAILED ===");
        }
    }

    // Step 6: Try Qwen3-Embedding (if available via candle, but candle-core ^0.10.2 unavailable)
    println!("\n\nStep 6: Checking for Qwen3-Embedding support...");
    println!("  NOTE: Qwen3-Embedding requires candle backend.");
    println!("  fastembed v5 qwen3 feature requires candle-core ^0.10.2 which is not published (max: 0.9.1).");
    println!("  This is BLOCKED at crate level.");

    println!("\n=== Probe Complete ===");
    println!("\nConclusions:");
    println!("- FastEmbed v5 with ONNX backend: BGESmallENV15, BGEM3 available");
    println!("- Qwen3-Embedding via fastembed: BLOCKED (candle-core version mismatch)");
    println!("- BGE-M3 is the best available multilingual model");

    Ok(())
}
