//! Technical verification for Windows Toast action buttons with Tauri v2
//!
//! This module is ONLY for technical verification - does NOT connect to real task flow.
//! Verification goals (from user):
//! 1. Windows crate Toast appears with hidden main window
//! 2. Toast has two action buttons: confirm / cancel
//! 3. Clicking confirm receives action=confirm and trace_id in Tauri backend
//! 4. Clicking cancel receives action=cancel (or proves no dispatch)
//! 5. App running case works
//! 6. Cold start case works
//! 7. Clicking action does NOT force show main window

use std::sync::RwLock;
use windows::core::HSTRING;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

/// AUMID for Cozmio - must match the app's registered AUMID
/// For Tauri apps, this is typically the identifier from tauri.conf.json
/// Development builds use the exe path, packaged builds use the registered AUMID
const AUMID: &str = "com.cozmio.app";

/// Global state to track verification results
static VERIFICATION_RESULT: RwLock<Option<VerificationResult>> = RwLock::new(None);

#[derive(Debug, Clone, serde::Serialize)]
pub struct VerificationResult {
    pub toast_shown: bool,
    pub action_buttons_count: usize,
    pub confirm_received: bool,
    pub confirm_trace_id: Option<String>,
    pub cancel_received: bool,
    pub cancel_trace_id: Option<String>,
    pub cold_start_received: bool,
    pub cold_start_url: Option<String>,
    pub main_window_forced_show: bool,
    pub error_message: Option<String>,
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self {
            toast_shown: false,
            action_buttons_count: 0,
            confirm_received: false,
            confirm_trace_id: None,
            cancel_received: false,
            cancel_trace_id: None,
            cold_start_received: false,
            cold_start_url: None,
            main_window_forced_show: false,
            error_message: None,
        }
    }
}

/// Generate Toast XML with action buttons
fn generate_toast_xml_with_actions(trace_id: &str) -> String {
    // Windows Toast XML with actions element
    // The action button arguments use the protocol URL scheme
    format!(
        r#"<toast activationType="protocol" launch="cozmio://action?trace_id={trace_id}">
            <visual>
                <binding template="ToastGeneric">
                    <text>Cozmio - 任务确认</text>
                    <text>点击确认执行，或取消</text>
                </binding>
            </visual>
            <actions>
                <action content="确认" activationType="protocol" arguments="cozmio://confirm?trace_id={trace_id}" />
                <action content="取消" activationType="protocol" arguments="cozmio://cancel?trace_id={trace_id}" />
            </actions>
        </toast>"#,
        trace_id = trace_id
    )
}

/// Generate simple Toast XML without action buttons (for comparison)
fn generate_simple_toast_xml() -> String {
    r#"<toast duration="long">
        <visual>
            <binding template="ToastGeneric">
                <text>Cozmio - Test</text>
                <text>Simple toast for verification</text>
            </binding>
        </visual>
    </toast>"#
        .to_string()
}

/// Generate result Toast XML without action buttons
fn generate_result_toast_xml(title: &str, body: &str) -> String {
    format!(
        r#"<toast duration="long">
            <visual>
                <binding template="ToastGeneric">
                    <text>{}</text>
                    <text>{}</text>
                </binding>
            </visual>
        </toast>"#,
        title, body
    )
}

/// Send a result notification Toast without action buttons
/// Used for completed/failed/interrupted session notifications
/// Returns: Ok(()) if toast shown, Err(message) if failed
pub fn send_result_toast(trace_id: &str, title: &str, body: &str) -> Result<(), String> {
    // Generate toast XML without actions
    let toast_xml = generate_result_toast_xml(title, body);

    // Create XML document from string
    let xml_doc = XmlDocument::new().map_err(|e| format!("Failed to create XmlDocument: {}", e))?;
    xml_doc
        .LoadXml(&HSTRING::from(&toast_xml))
        .map_err(|e| format!("Failed to load XML: {}", e))?;

    // Create toast notification
    let toast = ToastNotification::CreateToastNotification(&xml_doc)
        .map_err(|e| format!("Failed to create toast: {}", e))?;

    // Create notifier with AUMID
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))
        .map_err(|e| format!("Failed to create notifier with AUMID '{}': {}", AUMID, e))?;

    // Show the toast
    notifier
        .Show(&toast)
        .map_err(|e| format!("Failed to show toast: {}", e))?;

    log::info!(
        "Result toast sent trace_id={} title={} body={}",
        trace_id,
        title,
        body
    );

    Ok(())
}

/// Send a verification Toast with action buttons
/// Returns: Ok(()) if toast shown, Err(message) if failed
pub fn send_verification_toast(trace_id: &str) -> Result<(), String> {
    // Generate toast XML with actions
    let toast_xml = generate_toast_xml_with_actions(trace_id);

    // Create XML document from string
    let xml_doc = XmlDocument::new().map_err(|e| format!("Failed to create XmlDocument: {}", e))?;
    xml_doc
        .LoadXml(&HSTRING::from(&toast_xml))
        .map_err(|e| format!("Failed to load XML: {}", e))?;

    // Create toast notification
    let toast = ToastNotification::CreateToastNotification(&xml_doc)
        .map_err(|e| format!("Failed to create toast: {}", e))?;

    // Create notifier with AUMID
    // AUMID must match the app's registered AUMID for Toast activation to work
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))
        .map_err(|e| format!("Failed to create notifier with AUMID '{}': {}", AUMID, e))?;

    // Show the toast
    notifier
        .Show(&toast)
        .map_err(|e| format!("Failed to show toast: {}", e))?;

    // Update verification state
    {
        let mut result = VERIFICATION_RESULT.write().unwrap();
        *result = Some(VerificationResult {
            toast_shown: true,
            action_buttons_count: 2, // confirm + cancel
            ..Default::default()
        });
    }

    log::info!(
        "Verification toast sent with trace_id={}, xml={}",
        trace_id,
        toast_xml
    );

    Ok(())
}

/// Parse action from protocol URL
/// Returns: Some(("confirm", trace_id)) or Some(("cancel", trace_id)) or None
pub fn parse_protocol_url(url: &str) -> Option<(String, String)> {
    log::info!("Parsing protocol URL: {}", url);

    // URL format: cozmio://confirm?trace_id=xxx or cozmio://cancel?trace_id=xxx
    if url.starts_with("cozmio://confirm?trace_id=") {
        let trace_id = url.trim_start_matches("cozmio://confirm?trace_id=");
        Some(("confirm".to_string(), trace_id.to_string()))
    } else if url.starts_with("cozmio://cancel?trace_id=") {
        let trace_id = url.trim_start_matches("cozmio://cancel?trace_id=");
        Some(("cancel".to_string(), trace_id.to_string()))
    } else if url.starts_with("cozmio://action?trace_id=") {
        // Toast body click (not action button)
        let trace_id = url.trim_start_matches("cozmio://action?trace_id=");
        Some(("action".to_string(), trace_id.to_string()))
    } else {
        None
    }
}

/// Record that a confirm action was received
pub fn record_confirm_action(trace_id: &str) {
    log::info!("CONFIRM action received for trace_id={}", trace_id);
    let mut result = VERIFICATION_RESULT.write().unwrap();
    if let Some(ref mut r) = *result {
        r.confirm_received = true;
        r.confirm_trace_id = Some(trace_id.to_string());
    }
}

/// Record that a cancel action was received
pub fn record_cancel_action(trace_id: &str) {
    log::info!("CANCEL action received for trace_id={}", trace_id);
    let mut result = VERIFICATION_RESULT.write().unwrap();
    if let Some(ref mut r) = *result {
        r.cancel_received = true;
        r.cancel_trace_id = Some(trace_id.to_string());
    }
}

/// Record cold start URL reception
pub fn record_cold_start_url(url: &str) {
    log::info!("Cold start URL received: {}", url);
    let mut result = VERIFICATION_RESULT.write().unwrap();
    if let Some(ref mut r) = *result {
        r.cold_start_received = true;
        r.cold_start_url = Some(url.to_string());
    }
}

/// Mark that main window was NOT forced to show
pub fn record_no_main_window_show() {
    let mut result = VERIFICATION_RESULT.write().unwrap();
    if let Some(ref mut r) = *result {
        r.main_window_forced_show = false;
    }
}

/// Get current verification result
pub fn get_verification_result() -> Option<VerificationResult> {
    VERIFICATION_RESULT.read().unwrap().clone()
}

/// Reset verification state
pub fn reset_verification() {
    let mut result = VERIFICATION_RESULT.write().unwrap();
    *result = None;
}
