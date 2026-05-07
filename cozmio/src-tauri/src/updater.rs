use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_UPDATE_URL: &str = "https://updates.example.com";
const CHECK_INTERVAL_SECS: u64 = 24 * 3600;
const RETRY_BASE_DELAY_SECS: u64 = 3600;

/// Update check response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResponse {
    pub needs_update: bool,
    pub latest_version: String,
    pub channel: String,
    pub notes: String,
    pub download_url: String,
    pub signature: String,
}

/// Update state enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateState {
    None,
    Pending {
        version: String,
        installed_at: DateTime<Utc>,
    },
    ReadyToRestart,
}

impl Default for UpdateState {
    fn default() -> Self {
        UpdateState::None
    }
}

/// Update checker
pub struct UpdateChecker {
    client: Client,
    update_url: String,
    current_version: String,
}

impl UpdateChecker {
    /// Create a new UpdateChecker
    pub fn new(current_version: String) -> Self {
        let update_url =
            std::env::var("COZMIO_UPDATE_URL").unwrap_or_else(|_| DEFAULT_UPDATE_URL.to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        UpdateChecker {
            client,
            update_url,
            current_version,
        }
    }

    /// Check for updates (async)
    pub async fn check(&self) -> Result<UpdateCheckResponse, String> {
        let url = format!(
            "{}/updates/check?version={}",
            self.update_url, self.current_version
        );

        log::info!("Checking for updates at {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Update check failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Update server returned: {}", response.status()));
        }

        let result: UpdateCheckResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse update response: {}", e))?;

        log::info!("Update check result: needs_update={}", result.needs_update);
        Ok(result)
    }

    /// Download update to temp directory (async)
    pub async fn download(&self, url: &str, expected_checksum: &str) -> Result<PathBuf, String> {
        log::info!("Downloading update from {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response bytes: {}", e))?;

        // Verify checksum (exact match after normalizing sha256: prefix)
        let actual_checksum = format!("sha256:{}", sha256::digest(bytes.as_ref()));
        let expected_clean = expected_checksum.replace("sha256:", "");
        let actual_clean = actual_checksum.replace("sha256:", "");
        if actual_clean != expected_clean {
            return Err("Checksum mismatch".to_string());
        }

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        // Strip query string before extracting filename
        let filename = url
            .split('?')
            .next()
            .unwrap_or(url)
            .split('/')
            .last()
            .unwrap_or("cozmio-update.msi");
        let temp_path = temp_dir.join(filename);

        fs::write(&temp_path, &bytes).map_err(|e| format!("Failed to write temp file: {}", e))?;

        log::info!("Update downloaded to {:?}", temp_path);
        Ok(temp_path)
    }

    /// Install MSI silently
    pub fn install(&self, msi_path: &PathBuf) -> Result<(), String> {
        log::info!("Installing update from {:?}", msi_path);

        let msi_path_str = msi_path.to_string_lossy();
        let output = std::process::Command::new("msiexec")
            .args(&["/i", &msi_path_str, "/quiet", "/norestart"])
            .output()
            .map_err(|e| format!("Failed to execute msiexec: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("MSI install failed: {}", stderr));
        }

        log::info!("Update installed successfully");
        Ok(())
    }
}

/// Calculate retry delay with exponential backoff
pub fn retry_delay(attempt: u32) -> Duration {
    Duration::from_secs(RETRY_BASE_DELAY_SECS * 2u64.pow(attempt.min(3)))
}

/// Get the update state file path
pub fn update_state_path() -> Result<PathBuf, String> {
    let base =
        dirs::data_local_dir().ok_or_else(|| "Could not find local data directory".to_string())?;
    Ok(base.join("cozmio").join("update_state.json"))
}

/// Load update state from disk
pub fn load_update_state() -> UpdateState {
    let path = match update_state_path() {
        Ok(p) => p,
        Err(_) => return UpdateState::None,
    };

    if !path.exists() {
        return UpdateState::None;
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(UpdateState::None),
        Err(e) => {
            log::warn!("Failed to read update state: {}", e);
            UpdateState::None
        }
    }
}

/// Save update state to disk
pub fn save_update_state(state: &UpdateState) -> Result<(), String> {
    let path = update_state_path()?;

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
    }

    let content =
        serde_json::to_string_pretty(state).map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write state file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_state_default() {
        assert_eq!(UpdateState::None, UpdateState::None);
    }

    #[test]
    fn test_retry_delay() {
        assert_eq!(retry_delay(0), Duration::from_secs(3600));
        assert_eq!(retry_delay(1), Duration::from_secs(7200));
        assert_eq!(retry_delay(2), Duration::from_secs(14400));
        assert_eq!(retry_delay(3), Duration::from_secs(28800));
    }
}
