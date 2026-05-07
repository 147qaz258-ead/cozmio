use std::collections::HashMap;
use std::sync::RwLock;

/// Global state to store pending protocol actions (from cold start or Toast click)
static PENDING_PROTOCOL_ACTION: RwLock<Option<PendingProtocolAction>> = RwLock::new(None);

#[derive(Debug, Clone, serde::Serialize)]
pub struct PendingProtocolAction {
    pub action: String, // "confirm" or "cancel"
    pub trace_id: String,
    pub token: Option<String>,
}

impl PendingProtocolAction {
    /// Parse protocol URL and create a PendingProtocolAction
    /// Format: cozmio://confirm?trace_id=xxx or cozmio://cancel?trace_id=xxx
    pub fn from_url(url: &str) -> Option<Self> {
        log::info!("protocol_handler: Parsing URL: {}", url);

        // Expected formats:
        // - cozmio://confirm?trace_id=xxx
        // - cozmio://cancel?trace_id=xxx
        // - cozmio://action?trace_id=xxx (Toast body click)

        let url_clean = url.trim_start_matches("cozmio://");
        let (action, query) = url_clean.split_once('?')?;
        let normalized_action = action.trim_end_matches('/').to_string();

        let mut params: HashMap<String, String> = HashMap::new();
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(key.to_string(), urlencoding_decode(value));
            }
        }

        let trace_id = params.get("trace_id")?.clone();
        let token = params.get("token").cloned();

        Some(PendingProtocolAction {
            action: normalized_action,
            trace_id,
            token,
        })
    }
}

/// Decode URL-encoded string (%XX -> char)
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if c == '+' {
            // URL encoding uses + for space
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

/// Store a pending protocol action to be processed later
pub fn store_pending_action(action: PendingProtocolAction) {
    let mut guard = PENDING_PROTOCOL_ACTION.write().unwrap();
    log::info!("protocol_handler: Storing pending action: {:?}", action);
    *guard = Some(action);
}

/// Retrieve and clear any pending protocol action
pub fn take_pending_action() -> Option<PendingProtocolAction> {
    let mut guard = PENDING_PROTOCOL_ACTION.write().unwrap();
    let action = guard.take();
    if action.is_some() {
        log::info!("protocol_handler: Retrieved and cleared pending action");
    }
    action
}

/// Get current pending action without clearing it
pub fn get_pending_action() -> Option<PendingProtocolAction> {
    PENDING_PROTOCOL_ACTION.read().unwrap().clone()
}

/// Clear any pending protocol action
pub fn clear_pending_action() {
    let mut guard = PENDING_PROTOCOL_ACTION.write().unwrap();
    *guard = None;
}
