use cozmio_core::*;

#[test]
fn test_monitor_list_not_empty() {
    let monitors = get_monitors().unwrap();
    assert!(!monitors.is_empty(), "At least one monitor should exist");
}

#[test]
fn test_window_info_has_required_fields() {
    let windows = get_all_windows().unwrap();
    for w in windows {
        assert!(w.hwnd > 0, "HWND must be non-zero");
    }
}

#[test]
fn test_screenshot_capture() {
    let screenshot = Screenshot::capture(1).unwrap();
    assert!(
        !screenshot.image_base64.is_empty(),
        "Screenshot base64 must not be empty"
    );
    assert!(screenshot.width > 0, "Width must be non-zero");
    assert!(screenshot.height > 0, "Height must be non-zero");
    assert!(screenshot.timestamp > 0, "Timestamp must be non-zero");
}

#[test]
fn test_capture_all_produces_valid_json() {
    let result = capture_all(1).unwrap();
    let json = serde_json::to_string(&result).unwrap();
    assert!(!json.is_empty(), "JSON output must not be empty");
    assert!(
        json.contains("screenshot"),
        "JSON must contain screenshot field"
    );
    assert!(
        json.contains("foreground_window"),
        "JSON must contain foreground_window field"
    );
    assert!(
        json.contains("all_windows"),
        "JSON must contain all_windows field"
    );
}
