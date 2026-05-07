mod box_model_runtime;
mod config;
mod heartbeat;
mod model_provider;
mod protocol;
mod providers;
mod status;
mod worker;

use crate::config::Config;
use crate::worker::Worker;

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("===========================================");
    log::info!("cozmio-box-worker starting...");
    log::info!("===========================================");

    // Load configuration
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    log::info!(
        "Configuration loaded: worker_id={}, worker_type={}, relay_addr={}",
        config.worker_id,
        config.worker_type,
        config.relay_addr
    );

    // Create and start worker
    let mut worker = Worker::new(config.clone());

    // Connect to relay with retry loop (exponential backoff, max 30s)
    let mut backoff_secs: u64 = 1;
    let max_backoff_secs: u64 = 30;

    loop {
        match worker.connect() {
            Ok(()) => {
                log::info!("Worker is online and registered");
                break;
            }
            Err(e) => {
                log::error!("Failed to connect to relay: {}", e);
                log::info!(
                    "Retrying in {}s (max {}s)...",
                    backoff_secs,
                    max_backoff_secs
                );
                std::thread::sleep(std::time::Duration::from_secs(backoff_secs));

                // Exponential backoff with cap
                backoff_secs = (backoff_secs * 2).min(max_backoff_secs);

                // Create new worker instance for reconnect attempt
                worker = Worker::new(config.clone());
            }
        }
    }

    // Run the main loop to handle inference requests
    if let Err(e) = worker.run() {
        log::error!("Worker main loop error: {}", e);
    }

    log::info!("Worker stopped");

    // Wait for shutdown signal
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl+c");
            log::info!("Shutdown signal received, exiting...");
        });

    log::info!("cozmio-box-worker stopped");
}
