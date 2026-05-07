use crate::types::NotificationPending;
use std::sync::RwLock;
use windows::core::HSTRING;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

const AUMID: &str = "com.cozmio.app";
const CONFIRM_TOKEN_TTL_SECS: i64 = 10 * 60;

static NOTIFICATION_PENDING: RwLock<Option<NotificationPending>> = RwLock::new(None);

pub fn send_confirmation_notification(pending: &NotificationPending) -> Result<(), String> {
    let title = escape_xml("Cozmio - 任务确认");
    let content_text = &pending.content_text;
    let user_how = pending.user_how.as_deref();
    let body = if let Some(how) = user_how {
        format!(
            "{}\n\n我们会帮您：{}",
            truncate_for_notification(content_text, 150),
            truncate_for_notification(how, 50)
        )
    } else {
        truncate_for_notification(content_text, 200)
    };
    let confirm_url = pending.to_protocol_url("confirm");
    let cancel_url = pending.to_protocol_url("cancel");
    let launch_url = pending.to_protocol_url("action");
    let confirm_url_xml = escape_xml(&confirm_url);
    let cancel_url_xml = escape_xml(&cancel_url);
    let launch_url_xml = escape_xml(&launch_url);
    let toast_xml = format!(
        r#"<toast activationType="protocol" launch="{launch_url}" duration="long">
            <visual>
                <binding template="ToastGeneric">
                    <text>{title}</text>
                    <text>{body}</text>
                </binding>
            </visual>
            <actions>
                <action content="确认" activationType="protocol" arguments="{confirm_url}" />
                <action content="取消" activationType="protocol" arguments="{cancel_url}" />
            </actions>
        </toast>"#,
        launch_url = launch_url_xml,
        confirm_url = confirm_url_xml,
        cancel_url = cancel_url_xml,
    );

    show_toast_xml(&toast_xml)?;
    {
        let mut guard = NOTIFICATION_PENDING.write().unwrap();
        *guard = Some(pending.clone());
    }

    log::info!(
        "Confirmation toast sent trace_id={} token={} confirm_url={} cancel_url={}",
        pending.trace_id,
        pending.token.0,
        confirm_url,
        cancel_url
    );
    Ok(())
}

pub fn send_result_notification(
    trace_id: &str,
    content_text: &str,
    status: &str,
    result_text: Option<&str>,
    error_text: Option<&str>,
) -> Result<(), String> {
    let (title, body) = match status {
        "completed" => (
            escape_xml("Cozmio - 任务完成"),
            escape_xml(&truncate_for_notification(
                result_text.unwrap_or("任务已完成"),
                200,
            )),
        ),
        "failed" => (
            escape_xml("Cozmio - 任务失败"),
            escape_xml(&truncate_for_notification(
                error_text.unwrap_or("任务执行失败"),
                200,
            )),
        ),
        "interrupted" => (escape_xml("Cozmio - 任务中断"), escape_xml("任务已被中断")),
        _ => (
            escape_xml("Cozmio - 任务状态"),
            escape_xml("任务状态已更新"),
        ),
    };
    let content = escape_xml(&truncate_for_notification(content_text, 100));

    let toast_xml = format!(
        r#"<toast duration="long">
            <visual>
                <binding template="ToastGeneric">
                    <text>{title}</text>
                    <text>{content}</text>
                    <text>{body}</text>
                </binding>
            </visual>
        </toast>"#,
        content = content
    );

    show_toast_xml(&toast_xml)?;
    log::info!(
        "Result toast sent trace_id={} status={} title={} body={}",
        trace_id,
        status,
        title,
        body
    );
    Ok(())
}

pub fn consume_pending_notification(
    trace_id: &str,
    token: &str,
) -> Result<NotificationPending, String> {
    let now = chrono::Utc::now().timestamp();
    let mut guard = NOTIFICATION_PENDING.write().unwrap();
    let Some(pending) = guard.as_ref() else {
        return Err(String::from("No pending notification"));
    };

    if pending.created_at + CONFIRM_TOKEN_TTL_SECS < now {
        log::warn!(
            "Notification token expired trace_id={} created_at={}",
            pending.trace_id,
            pending.created_at
        );
        *guard = None;
        return Err(String::from("Notification token expired"));
    }

    if pending.trace_id != trace_id || pending.token.0 != token {
        log::warn!(
            "Notification token validation failed expected_trace_id={} got_trace_id={} expected_token={} got_token={}",
            pending.trace_id,
            trace_id,
            pending.token.0,
            token
        );
        return Err(String::from("Notification token validation failed"));
    }

    let consumed = guard.take().expect("pending notification disappeared");
    log::info!(
        "Notification token consumed trace_id={} token={}",
        consumed.trace_id,
        consumed.token.0
    );
    Ok(consumed)
}

pub fn clear_pending_notification() {
    let mut guard = NOTIFICATION_PENDING.write().unwrap();
    *guard = None;
}

fn show_toast_xml(xml: &str) -> Result<(), String> {
    let xml_doc = XmlDocument::new().map_err(|e| format!("Failed to create XmlDocument: {}", e))?;
    xml_doc
        .LoadXml(&HSTRING::from(xml))
        .map_err(|e| format!("Failed to load toast XML: {}", e))?;

    let toast = ToastNotification::CreateToastNotification(&xml_doc)
        .map_err(|e| format!("Failed to create toast: {}", e))?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))
        .map_err(|e| format!("Failed to create notifier with AUMID '{}': {}", AUMID, e))?;
    notifier
        .Show(&toast)
        .map_err(|e| format!("Failed to show toast: {}", e))
}

fn truncate_for_notification(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        text.chars().take(max_chars - 3).collect::<String>() + "..."
    }
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
