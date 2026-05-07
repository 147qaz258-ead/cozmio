use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration structure for cozmio application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ollama_url: String,
    pub model_name: String,
    pub poll_interval_secs: u64,
    pub window_change_detection: bool,
    pub execute_auto: bool,
    pub request_use_native_dialog: bool,
    pub execute_delay_secs: u64,
    #[serde(default = "default_true")]
    pub memory_flywheel_enabled: bool,
    #[serde(default = "default_memory_mode")]
    pub memory_maintenance_mode: String,  // "local" | "cloud" | "executor"
    #[serde(default)]
    pub consolidation_model_name: Option<String>,
    pub last_check_at: Option<String>, // 新增: ISO8601 时间戳
    pub update_channel: String,        // 新增: "stable" 或 "beta"
}

fn default_true() -> bool {
    true
}

fn default_memory_mode() -> String {
    "local".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ollama_url: "http://localhost:11434".to_string(),
            model_name: "llava".to_string(),
            poll_interval_secs: 3,
            window_change_detection: true,
            execute_auto: true,
            request_use_native_dialog: true,
            execute_delay_secs: 1,
            memory_flywheel_enabled: true,
            memory_maintenance_mode: "local".to_string(),
            consolidation_model_name: None,
            last_check_at: None,                  // 新增
            update_channel: "stable".to_string(), // 新增
        }
    }
}

impl Config {
    /// Returns the configuration directory path
    fn config_dir() -> Result<PathBuf, String> {
        let base = dirs::data_local_dir()
            .ok_or_else(|| "Could not find local data directory".to_string())?;
        Ok(base.join("cozmio"))
    }

    /// Returns the full configuration file path
    fn config_path() -> Result<PathBuf, String> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    /// Load configuration from file
    /// Returns Default config if file doesn't exist or is invalid
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Config::default());
        }

        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir()?;
        let path = Self::config_path()?;

        // Create directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.model_name, "llava");
        assert_eq!(config.poll_interval_secs, 3);
        assert!(config.window_change_detection);
        assert!(config.execute_auto);
        assert!(config.request_use_native_dialog);
        assert_eq!(config.execute_delay_secs, 1);
        assert!(config.memory_flywheel_enabled);
        assert!(config.last_check_at.is_none());
        assert_eq!(config.update_channel, "stable");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.ollama_url, config.ollama_url);
        assert_eq!(parsed.model_name, config.model_name);
        assert_eq!(parsed.poll_interval_secs, config.poll_interval_secs);
    }

    #[test]
    fn test_config_path_creation() {
        // Test that config_dir returns a valid path
        let dir = Config::config_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert_eq!(path.file_name().unwrap(), "cozmio");
    }

    #[test]
    fn test_load_and_save() {
        // Use a temp location for testing
        env::set_var("LOCALAPPDATA", env::temp_dir().to_str().unwrap());

        let config = Config {
            ollama_url: "http://test:11434".to_string(),
            model_name: "test_model".to_string(),
            poll_interval_secs: 10,
            window_change_detection: false,
            execute_auto: false,
            request_use_native_dialog: false,
            execute_delay_secs: 5,
            memory_flywheel_enabled: true,
            memory_maintenance_mode: "local".to_string(),
            consolidation_model_name: None,
            last_check_at: Some("2024-01-01T00:00:00Z".to_string()),
            update_channel: "beta".to_string(),
        };

        // Save
        let save_result = config.save();
        assert!(
            save_result.is_ok(),
            "Save should succeed, got: {:?}",
            save_result.err()
        );

        // Load
        let loaded = Config::load();
        assert!(
            loaded.is_ok(),
            "Load should succeed, got: {:?}",
            loaded.err()
        );

        let loaded = loaded.unwrap();
        assert_eq!(loaded.ollama_url, "http://test:11434");
        assert_eq!(loaded.model_name, "test_model");
        assert_eq!(loaded.poll_interval_secs, 10);
        assert!(!loaded.window_change_detection);
        assert!(!loaded.execute_auto);
        assert!(!loaded.request_use_native_dialog);
        assert_eq!(loaded.execute_delay_secs, 5);
        assert_eq!(
            loaded.last_check_at,
            Some("2024-01-01T00:00:00Z".to_string())
        );
        assert_eq!(loaded.update_channel, "beta");
    }
}
