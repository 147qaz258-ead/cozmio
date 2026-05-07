use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub worker_id: String,
    pub worker_type: String,
    pub relay_addr: String,
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,
}

fn default_heartbeat_interval() -> u64 {
    30
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Err(format!("Config file not found at: {}", config_path.display()).into());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;

        log::info!(
            "Loaded config from {}: worker_id={}, relay_addr={}",
            config_path.display(),
            config.worker_id,
            config.relay_addr
        );

        Ok(config)
    }

    fn config_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            Ok(Path::new("/opt/cozmio/config/box-worker.toml").to_path_buf())
        }
        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA")?;
            Ok(Path::new(&local_app_data)
                .join("cozmio")
                .join("config")
                .join("box-worker.toml"))
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err("Unsupported platform".into())
        }
    }
}

#[cfg(target_os = "linux")]
pub fn get_deployment_paths() -> Paths {
    Paths {
        binary: "/home/pi/cozmio/box-worker",
        config: "/home/pi/cozmio/config/box-worker.toml",
        logs: "/home/pi/cozmio/logs/box-worker.log",
        service: "/home/pi/cozmio/cozmio-box-worker.service",
    }
}

#[cfg(target_os = "windows")]
pub fn get_deployment_paths() -> Paths {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    Paths {
        binary: format!("{}/cozmio/box-worker.exe", local_app_data),
        config: format!("{}/cozmio/config/box-worker.toml", local_app_data),
        logs: format!("{}/cozmio/logs/box-worker.log", local_app_data),
        service: "".to_string(), // Windows uses a different service mechanism
    }
}

#[derive(Debug, Clone)]
pub struct Paths {
    pub binary: String,
    pub config: String,
    pub logs: String,
    pub service: String,
}
