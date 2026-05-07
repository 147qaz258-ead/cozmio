use anyhow::Result;
use clap::{Parser, Subcommand};
use cozmio_core::capture_all;
use cozmio_memory::db::Database;
use cozmio_memory::memory_events::MemoryEventsStore;
use cozmio_memory::search::SearchEngine;
use cozmio_model::{
    ask_model_sync, discover_first_model_sync, parse_intervention_result, InterventionMode,
};
use relay_client::client::RelayClient;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "cozmio-cli")]
#[command(about = "CLI interface for cozmio")]
enum Cli {
    /// Window management commands
    #[command(subcommand)]
    Window(WindowCommands),
    /// Model interaction commands
    #[command(subcommand)]
    Model(ModelCommands),
    /// Relay interaction commands
    #[command(subcommand)]
    Relay(RelayCommands),
    /// Memory database commands
    #[command(subcommand)]
    Memory(MemoryCommands),
    /// Configuration management commands
    #[command(subcommand)]
    Config(ConfigCommands),
    /// Run the monitoring loop
    #[command(subcommand)]
    Run(RunCommands),
    /// Check cozmio runtime status
    Status,
}

#[derive(Subcommand)]
enum WindowCommands {
    /// Capture screenshot of foreground window and output JSON
    Capture,
    /// List all visible windows in a table
    List,
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List available models from Ollama
    List,
    /// Call the model with text after capturing screenshot
    Call { text: String },
}

#[derive(Subcommand)]
enum RelayCommands {
    /// Dispatch a task to the relay agent
    Dispatch { task: String },
    /// Get status of a relay session
    Status { session_id: String },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Show memory database statistics
    Stats,
    /// Search memory database
    Search { query: String },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set { key: String, value: String },
}

#[derive(Subcommand)]
enum RunCommands {
    /// Capture current window and output judgment (single-shot)
    Once,
    /// Run the full monitoring loop (daemon mode)
    Daemon,
}

#[derive(Serialize)]
struct WindowCaptureOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    screenshot_bytes: Option<String>,
    window_info: cozmio_core::WindowInfo,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Window(cmd) => match cmd {
            WindowCommands::Capture => cmd_window_capture()?,
            WindowCommands::List => cmd_window_list()?,
        },
        Cli::Model(cmd) => match cmd {
            ModelCommands::List => cmd_model_list()?,
            ModelCommands::Call { text } => cmd_model_call(&text)?,
        },
        Cli::Relay(cmd) => match cmd {
            RelayCommands::Dispatch { task } => cmd_relay_dispatch(&task)?,
            RelayCommands::Status { session_id } => cmd_relay_status(&session_id)?,
        },
        Cli::Memory(cmd) => match cmd {
            MemoryCommands::Stats => cmd_memory_stats()?,
            MemoryCommands::Search { query } => cmd_memory_search(&query)?,
        },
        Cli::Config(cmd) => match cmd {
            ConfigCommands::Show => cmd_config_show()?,
            ConfigCommands::Set { key, value } => cmd_config_set(&key, &value)?,
        },
        Cli::Run(cmd) => match cmd {
            RunCommands::Once => cmd_run_once()?,
            RunCommands::Daemon => cmd_run_daemon()?,
        },
        Cli::Status => cmd_status()?,
    }

    Ok(())
}

fn cmd_window_capture() -> Result<()> {
    let result = capture_all(0)?;

    let output = WindowCaptureOutput {
        screenshot_bytes: result.screenshot.map(|s| s.image_base64),
        window_info: result.foreground_window.unwrap_or(cozmio_core::WindowInfo {
            hwnd: 0,
            title: String::new(),
            process_name: String::new(),
            process_id: 0,
            monitor_index: 0,
            rect: cozmio_core::Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            is_foreground: false,
            is_visible: false,
            z_order: 0,
        }),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn cmd_window_list() -> Result<()> {
    let result = capture_all(0)?;

    println!(
        "{:<10} {:<40} {:<25} {:<8} {:<8}",
        "HWND", "TITLE", "PROCESS", "PID", "FOREGROUND"
    );
    println!("{}", "-".repeat(100));

    for window in result.all_windows.windows.iter() {
        let title = if window.title.len() > 38 {
            format!("{}...", &window.title[..38])
        } else {
            window.title.clone()
        };
        let process = if window.process_name.len() > 23 {
            format!("{}...", &window.process_name[..23])
        } else {
            window.process_name.clone()
        };
        println!(
            "{:<10} {:<40} {:<25} {:<8} {:<8}",
            window.hwnd,
            title,
            process,
            window.process_id,
            if window.is_foreground { "yes" } else { "no" }
        );
    }

    println!("\nTotal: {} windows", result.all_windows.count);
    Ok(())
}

fn cmd_model_list() -> Result<()> {
    let model_name = discover_first_model_sync("http://localhost:11434")?;
    println!("Model: {}", model_name);
    println!("URL: http://localhost:11434");
    Ok(())
}

fn cmd_model_call(text: &str) -> Result<()> {
    // Capture screenshot
    let capture_result = capture_all(0)?;

    let screenshot_base64 = match &capture_result.screenshot {
        Some(s) => s.image_base64.clone(),
        None => {
            anyhow::bail!("Failed to capture screenshot");
        }
    };

    let window_info = capture_result
        .foreground_window
        .ok_or_else(|| anyhow::anyhow!("No foreground window found"))?;

    // Call model
    let model_name = discover_first_model_sync("http://localhost:11434")?;
    let raw_output = ask_model_sync(
        &model_name,
        &screenshot_base64,
        &window_info.title,
        &window_info.process_name,
        text,
    )?;

    // Parse and output result
    let result = parse_intervention_result(&raw_output)?;
    let output = InterventionOutput {
        model: model_name,
        mode: result.mode,
        reason: result.reason,
        raw_output: result.raw_output,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[derive(Serialize)]
struct InterventionOutput {
    model: String,
    mode: InterventionMode,
    reason: String,
    raw_output: String,
}

// === Relay Commands ===

fn cmd_relay_dispatch(task: &str) -> Result<()> {
    let client = RelayClient::connect("127.0.0.1:8741")?;
    let session_id = client.dispatch("cozmio", "suggestion", task)?;
    println!("{}", serde_json::json!({ "session_id": session_id }));
    Ok(())
}

fn cmd_relay_status(session_id: &str) -> Result<()> {
    let client = RelayClient::connect("127.0.0.1:8741")?;
    let status = client.status(session_id)?;
    let output = serde_json::json!({
        "session_id": status.session_id,
        "status": status.status,
        "started_at": status.started_at,
        "updated_at": status.updated_at,
        "duration_secs": status.duration_secs,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// === Memory Commands ===

#[derive(Serialize)]
struct MemoryStats {
    total_events: i64,
    total_slices: i64,
}

fn cmd_memory_stats() -> Result<()> {
    let db_path = Database::memory_dir().join("memory.db");
    if !db_path.exists() {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Database not found",
                "path": db_path.to_string_lossy()
            }))?
        );
        return Ok(());
    }

    let db =
        Database::new(&db_path).map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;

    let events_store = MemoryEventsStore::new(&db);

    let total_events = events_store
        .count()
        .map_err(|e| anyhow::anyhow!("Failed to count events: {}", e))?;

    // Count slices via direct SQL
    let conn = db.conn.lock().unwrap();
    let total_slices: i64 = conn
        .query_row("SELECT COUNT(*) FROM context_slices", [], |row| row.get(0))
        .unwrap_or(0);
    drop(conn);

    let stats = MemoryStats {
        total_events,
        total_slices,
    };

    println!("{}", serde_json::to_string_pretty(&stats)?);
    Ok(())
}

fn cmd_memory_search(query: &str) -> Result<()> {
    let db_path = Database::memory_dir().join("memory.db");
    if !db_path.exists() {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "error": "Database not found",
                "path": db_path.to_string_lossy()
            }))?
        );
        return Ok(());
    }

    let db =
        Database::new(&db_path).map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;

    let search_engine = SearchEngine::new(&db, None);
    let search_query = cozmio_memory::search::SearchQuery {
        text: Some(query.to_string()),
        time_range: None,
        thread_id: None,
        limit: 10,
    };

    let results = search_engine
        .search(&search_query)
        .map_err(|e| anyhow::anyhow!("Search failed: {}", e))?;

    // Manually serialize since SearchResult doesn't implement Serialize
    let events_json: Vec<serde_json::Value> = results
        .events
        .iter()
        .map(|e| {
            serde_json::json!({
                "event_id": e.event_id,
                "score": e.score,
                "source": e.source,
                "content": e.content,
                "window_title": e.window_title,
                "timestamp": e.timestamp,
                "evidence_source": e.evidence_source,
                "thread_id": e.thread_id,
            })
        })
        .collect();

    let output = serde_json::json!({
        "events": events_json,
        "total_fts": results.total_fts,
        "total_vec": results.total_vec,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// === Config Commands ===

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CliConfig {
    ollama_url: String,
    model_name: String,
    poll_interval_secs: u64,
    window_change_detection: bool,
    execute_auto: bool,
    #[serde(default)]
    request_use_native_dialog: bool,
    #[serde(default)]
    execute_delay_secs: u64,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            model_name: "llava".to_string(),
            poll_interval_secs: 3,
            window_change_detection: true,
            execute_auto: true,
            request_use_native_dialog: true,
            execute_delay_secs: 1,
        }
    }
}

impl CliConfig {
    fn config_path() -> Result<PathBuf, String> {
        let base = dirs::data_local_dir()
            .ok_or_else(|| "Could not find local data directory".to_string())?;
        Ok(base.join("cozmio").join("config.json"))
    }

    fn load() -> Result<Self, String> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(CliConfig::default());
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))
    }

    fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))
    }
}

fn cmd_config_show() -> Result<()> {
    let config = CliConfig::load().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

fn cmd_config_set(key: &str, value: &str) -> Result<()> {
    let mut config = CliConfig::load().map_err(|e| anyhow::anyhow!("{}", e))?;

    match key {
        "ollama_url" => config.ollama_url = value.to_string(),
        "model_name" => config.model_name = value.to_string(),
        "poll_interval_secs" => {
            config.poll_interval_secs = value
                .parse()
                .map_err(|_| anyhow::anyhow!("poll_interval_secs must be a number"))?;
        }
        "window_change_detection" => {
            config.window_change_detection = value
                .parse()
                .map_err(|_| anyhow::anyhow!("window_change_detection must be true or false"))?;
        }
        "execute_auto" => {
            config.execute_auto = value
                .parse()
                .map_err(|_| anyhow::anyhow!("execute_auto must be true or false"))?;
        }
        "execute_delay_secs" => {
            config.execute_delay_secs = value
                .parse()
                .map_err(|_| anyhow::anyhow!("execute_delay_secs must be a number"))?;
        }
        _ => anyhow::bail!("Unknown config key: {}", key),
    }

    config.save().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("Config updated successfully");
    Ok(())
}

// === Run Commands ===

fn cmd_run_once() -> Result<()> {
    // Capture current foreground window
    let capture_result = capture_all(0)?;

    let screenshot_base64 = match &capture_result.screenshot {
        Some(s) => s.image_base64.clone(),
        None => {
            anyhow::bail!("Failed to capture screenshot");
        }
    };

    let window_info = capture_result
        .foreground_window
        .ok_or_else(|| anyhow::anyhow!("No foreground window found"))?;

    // Discover model
    let model_name = discover_first_model_sync("http://localhost:11434")?;

    // Call model
    let raw_output = ask_model_sync(
        &model_name,
        &screenshot_base64,
        &window_info.title,
        &window_info.process_name,
        "",
    )?;

    // Parse result
    let result = parse_intervention_result(&raw_output)?;

    // Output as JSON using serde_json::json! macro
    let output = serde_json::json!({
        "mode": result.mode,
        "reason": result.reason,
        "raw_output": result.raw_output,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn cmd_run_daemon() -> Result<()> {
    println!("Daemon mode not yet implemented");
    println!("This would run the full monitoring loop without GUI.");
    println!("Use 'cozmio-cli run --once' for single-shot capture.");
    Ok(())
}

// === Status Command ===

#[derive(Serialize)]
struct StatusOutput {
    process_running: bool,
    runtime_state: Option<RuntimeStateInfo>,
}

#[derive(Serialize)]
struct RuntimeStateInfo {
    running_state: String,
    loop_tick_count: u64,
    last_loop_at: Option<String>,
    last_popup_requested_at: Option<String>,
    popup_count: u64,
}

fn cmd_status() -> Result<()> {
    // Check if cozmio.exe process is running
    let process_running = is_process_running("cozmio.exe");

    // Try to read runtime state file
    let runtime_state = read_runtime_state_file();

    let output = StatusOutput {
        process_running,
        runtime_state,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn is_process_running(process_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}", process_name)])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.lines().any(|line| line.contains(process_name));
        }
    }
    false
}

fn read_runtime_state_file() -> Option<RuntimeStateInfo> {
    let path = dirs::data_local_dir()?
        .join("cozmio")
        .join("runtime_state.json");

    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;
    let state: serde_json::Value = serde_json::from_str(&content).ok()?;

    Some(RuntimeStateInfo {
        running_state: state.get("running_state")?.as_str()?.to_string(),
        loop_tick_count: state.get("loop_tick_count")?.as_u64()?,
        last_loop_at: state
            .get("last_loop_at")
            .and_then(|v| v.as_str().map(String::from)),
        last_popup_requested_at: state
            .get("last_popup_requested_at")
            .and_then(|v| v.as_str().map(String::from)),
        popup_count: state.get("popup_count")?.as_u64()?,
    })
}
