use cozmio_core::{capture_all, get_monitors};
use serde_json;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--monitors".to_string()) {
        // List monitors and exit
        match get_monitors() {
            Ok(monitors) => {
                println!("Available monitors:");
                for m in monitors {
                    println!("  {}: {}x{}+{}+{}", m.index, m.width, m.height, m.x, m.y);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Default: capture all info from monitor 1
    let monitor_index: u32 = args
        .iter()
        .position(|a| a == "--monitor")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    match capture_all(monitor_index) {
        Ok(result) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("Failed to serialize result")
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
