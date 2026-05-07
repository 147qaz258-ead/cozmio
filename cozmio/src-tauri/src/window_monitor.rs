use cozmio_core::{capture_all, CaptureAllResult, WindowInfo};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferedEntry {
    pub window_title: String,
    pub process_name: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessBuffer {
    entries: VecDeque<BufferedEntry>,
    capacity: usize,
}

impl ProcessBuffer {
    pub fn new(capacity: usize) -> Self {
        ProcessBuffer {
            entries: VecDeque::new(),
            capacity,
        }
    }

    pub fn push(&mut self, entry: BufferedEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessContext {
    pub stay_duration_seconds: u32,
    pub switches_in_last_minute: u32,
    pub rapid_switch_intervals_under_5s: u32,
    pub foreground_changed_within_5s: bool,
}

#[derive(Debug, Clone)]
pub struct WindowSnapshot {
    pub screenshot_base64: String,
    pub screenshot_width: u32,
    pub screenshot_height: u32,
    pub window_info: WindowInfo,
    pub timestamp: i64,
}

#[derive(Debug)]
pub struct WindowMonitor {
    last_window_title: String,
    buffer: ProcessBuffer,
}

impl WindowMonitor {
    pub fn new() -> Self {
        WindowMonitor {
            last_window_title: String::new(),
            buffer: ProcessBuffer::new(1000),
        }
    }

    pub fn compute_context(
        &self,
        window_title: &str,
        _process_name: &str,
        timestamp: i64,
    ) -> ProcessContext {
        // Calculate using OLD buffer (before current entry is added)

        // stay_duration_seconds: find last entry for current window in buffer (NOT current), compute diff
        let stay_duration_seconds = if let Some(last_entry) = self
            .buffer
            .entries
            .iter()
            .filter(|e| e.window_title == window_title)
            .last()
        {
            (timestamp - last_entry.timestamp) as u32
        } else {
            0
        };

        // switches_in_last_minute: scan buffer (excluding current), count entries with different window_titles in last 60s
        let sixty_secs_ago = timestamp - 60;
        let mut switches_in_last_minute = 0u32;
        let mut prev_title: Option<&str> = None;
        for entry in self
            .buffer
            .entries
            .iter()
            .filter(|e| e.timestamp >= sixty_secs_ago)
        {
            if let Some(prev) = prev_title {
                if entry.window_title != prev {
                    switches_in_last_minute += 1;
                }
            }
            prev_title = Some(&entry.window_title);
        }

        let mut rapid_switch_intervals_under_5s = 0u32;
        let mut switch_times: Vec<i64> = Vec::new();
        let mut prev_title_for_switch: Option<&str> = None;
        for entry in self
            .buffer
            .entries
            .iter()
            .filter(|e| e.timestamp >= sixty_secs_ago)
        {
            if let Some(prev) = prev_title_for_switch {
                if entry.window_title != prev {
                    switch_times.push(entry.timestamp);
                }
            }
            prev_title_for_switch = Some(&entry.window_title);
        }
        if switch_times.len() >= 2 {
            for i in 1..switch_times.len() {
                if switch_times[i] - switch_times[i - 1] < 5 {
                    rapid_switch_intervals_under_5s += 1;
                }
            }
        }

        let foreground_changed_within_5s = if let Some(last_entry) = self.buffer.entries.back() {
            last_entry.window_title != window_title && (timestamp - last_entry.timestamp) < 5
        } else {
            false
        };

        ProcessContext {
            stay_duration_seconds,
            switches_in_last_minute,
            rapid_switch_intervals_under_5s,
            foreground_changed_within_5s,
        }
    }

    pub fn push_snapshot(&mut self, window_title: String, process_name: String, timestamp: i64) {
        self.buffer.push(BufferedEntry {
            window_title,
            process_name,
            timestamp,
        });
    }

    pub fn capture(&self) -> Result<WindowSnapshot, String> {
        let result: CaptureAllResult = capture_all(1).map_err(|e| e.to_string())?;

        let screenshot = result.screenshot.ok_or("No screenshot available")?;
        let window_info = result.foreground_window.ok_or("No foreground window")?;

        Ok(WindowSnapshot {
            screenshot_base64: screenshot.image_base64,
            screenshot_width: screenshot.width,
            screenshot_height: screenshot.height,
            window_info,
            timestamp: result.timestamp,
        })
    }

    pub fn has_changed(&self, snapshot: &WindowSnapshot) -> bool {
        snapshot.window_info.title != self.last_window_title
    }

    pub fn update_last_title(&mut self, title: &str) {
        self.last_window_title = title.to_string();
    }
}

impl Default for WindowMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_monitor_new() {
        let monitor = WindowMonitor::new();
        assert!(monitor.last_window_title.is_empty());
    }

    #[test]
    fn test_has_changed_initial() {
        let monitor = WindowMonitor::new();
        let snapshot = WindowSnapshot {
            screenshot_base64: String::new(),
            screenshot_width: 1920,
            screenshot_height: 1080,
            window_info: WindowInfo {
                hwnd: 1,
                title: "Test Window".to_string(),
                process_name: "test.exe".to_string(),
                process_id: 1234,
                monitor_index: 1,
                rect: cozmio_core::Rect {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                is_foreground: true,
                is_visible: true,
                z_order: 0,
            },
            timestamp: 1234567890,
        };
        // Initially empty, so any title should be considered "changed"
        assert!(monitor.has_changed(&snapshot));
    }

    #[test]
    fn test_has_changed_same_title() {
        let mut monitor = WindowMonitor::new();
        monitor.update_last_title("Test Window");

        let snapshot = WindowSnapshot {
            screenshot_base64: String::new(),
            screenshot_width: 1920,
            screenshot_height: 1080,
            window_info: WindowInfo {
                hwnd: 1,
                title: "Test Window".to_string(),
                process_name: "test.exe".to_string(),
                process_id: 1234,
                monitor_index: 1,
                rect: cozmio_core::Rect {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                is_foreground: true,
                is_visible: true,
                z_order: 0,
            },
            timestamp: 1234567890,
        };

        assert!(!monitor.has_changed(&snapshot));
    }

    #[test]
    fn test_has_changed_different_title() {
        let mut monitor = WindowMonitor::new();
        monitor.update_last_title("Old Window");

        let snapshot = WindowSnapshot {
            screenshot_base64: String::new(),
            screenshot_width: 1920,
            screenshot_height: 1080,
            window_info: WindowInfo {
                hwnd: 1,
                title: "New Window".to_string(),
                process_name: "test.exe".to_string(),
                process_id: 1234,
                monitor_index: 1,
                rect: cozmio_core::Rect {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                is_foreground: true,
                is_visible: true,
                z_order: 0,
            },
            timestamp: 1234567890,
        };

        assert!(monitor.has_changed(&snapshot));
    }

    #[test]
    fn test_update_last_title() {
        let mut monitor = WindowMonitor::new();
        assert!(monitor.last_window_title.is_empty());

        monitor.update_last_title("New Title");
        assert_eq!(monitor.last_window_title, "New Title");
    }

    #[test]
    fn test_compute_context_stay_duration() {
        let mut monitor = WindowMonitor::new();

        // Push some entries
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1010);

        // Compute context for Window A at timestamp 1020
        // Buffer has [Window A@1000, Window B@1010]
        // Most recent Window A in buffer is at 1000, so stay_duration = 1020 - 1000 = 20
        let ctx = monitor.compute_context("Window A", "proc.exe", 1020);
        assert_eq!(ctx.stay_duration_seconds, 20);
    }

    #[test]
    fn test_compute_context_stay_duration_no_history() {
        let monitor = WindowMonitor::new();

        // No history for Window C, stay_duration should be 0
        let ctx = monitor.compute_context("Window C", "proc.exe", 1000);
        assert_eq!(ctx.stay_duration_seconds, 0);
    }

    #[test]
    fn test_compute_context_switches_in_last_minute() {
        let mut monitor = WindowMonitor::new();

        // Build buffer: A@1000, B@1010, A@1020, B@1030 (4 entries over 30 seconds)
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1010);
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1020);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1030);

        // At timestamp 1040, looking at last 60 seconds (timestamp >= 980)
        // Entries in window: all 4 (all within 60s)
        // Switches: A->B (1), B->A (2), A->B (3) = 3 switches
        let ctx = monitor.compute_context("Window C", "proc.exe", 1040);
        assert_eq!(ctx.switches_in_last_minute, 3);
    }

    #[test]
    fn test_compute_context_switches_only_count_different_titles() {
        let mut monitor = WindowMonitor::new();

        // Build buffer: A@1000, A@1010, A@1020 (same window, no switches)
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1010);
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1020);

        // No switches between different windows
        let ctx = monitor.compute_context("Window B", "proc.exe", 1030);
        assert_eq!(ctx.switches_in_last_minute, 0);
    }

    #[test]
    fn test_compute_context_counts_rapid_switch_intervals_under_5s() {
        let mut monitor = WindowMonitor::new();

        // Build rapid switching facts: A@1000, B@1002, A@1004, B@1006
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1002);
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1004);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1006);

        let ctx = monitor.compute_context("Window A", "proc.exe", 1010);
        assert_eq!(ctx.rapid_switch_intervals_under_5s, 2);
    }

    #[test]
    fn test_compute_context_counts_zero_rapid_switch_intervals_for_slow_switches() {
        let mut monitor = WindowMonitor::new();

        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1010);
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1020);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1030);

        let ctx = monitor.compute_context("Window A", "proc.exe", 1040);
        assert_eq!(ctx.rapid_switch_intervals_under_5s, 0);
    }

    #[test]
    fn test_compute_context_foreground_changed_within_5s_true() {
        let mut monitor = WindowMonitor::new();

        // Buffer: A@1000, B@1010
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1010);

        let ctx = monitor.compute_context("Window A", "proc.exe", 1012);
        assert!(ctx.foreground_changed_within_5s);
    }

    #[test]
    fn test_compute_context_foreground_changed_within_5s_false_too_slow() {
        let mut monitor = WindowMonitor::new();

        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1010);

        let ctx = monitor.compute_context("Window A", "proc.exe", 1020);
        assert!(!ctx.foreground_changed_within_5s);
    }

    #[test]
    fn test_compute_context_foreground_changed_within_5s_false_same_window() {
        let mut monitor = WindowMonitor::new();

        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1010);

        let ctx = monitor.compute_context("Window B", "proc.exe", 1012);
        assert!(!ctx.foreground_changed_within_5s);
    }

    #[test]
    fn test_compute_context_foreground_changed_within_5s_false_empty_buffer() {
        let monitor = WindowMonitor::new();
        let ctx = monitor.compute_context("Window A", "proc.exe", 1000);
        assert!(!ctx.foreground_changed_within_5s);
    }

    #[test]
    fn test_compute_context_foreground_changed_within_5s_true_single_entry() {
        let mut monitor = WindowMonitor::new();

        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1010);

        let ctx = monitor.compute_context("Window A", "proc.exe", 1012);
        assert!(ctx.foreground_changed_within_5s);
    }

    #[test]
    fn test_compute_context_rapid_switch_interval_uses_5s_threshold() {
        let mut monitor = WindowMonitor::new();

        // A@1000, B@1005, A@1010, B@1015
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1000);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1005);
        monitor.push_snapshot("Window A".to_string(), "proc.exe".to_string(), 1010);
        monitor.push_snapshot("Window B".to_string(), "proc.exe".to_string(), 1015);

        let ctx = monitor.compute_context("Window A", "proc.exe", 1020);
        assert_eq!(ctx.rapid_switch_intervals_under_5s, 0);
    }
}
