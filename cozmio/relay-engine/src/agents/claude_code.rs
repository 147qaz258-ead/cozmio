use crate::agent::ExecutionAgent;
use crate::error::{Error, Result};
use crate::session::{
    ExecutionResult, LogLevel, ProgressEntry, SessionId, SessionManager, SessionStatus,
};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub struct ClaudeCodeConnector {
    cli_path: String,
    sessions: Arc<SessionManager>,
}

impl ClaudeCodeConnector {
    pub fn new(cli_path: String, sessions: Arc<SessionManager>) -> Self {
        ClaudeCodeConnector { cli_path, sessions }
    }
}

impl ExecutionAgent for ClaudeCodeConnector {
    fn name(&self) -> &str {
        "claude-code"
    }

    /// NOTE: This method returns immediately after spawning the task. The task runs asynchronously.
    fn dispatch(&self, session_id: SessionId, task: String) -> Result<()> {
        let cli_path = self.cli_path.clone();
        let sessions = self.sessions.clone();
        self.sessions
            .update_status(&session_id, SessionStatus::Running);

        thread::spawn(move || {
            let start = Instant::now();
            let cli_task = normalize_task_for_cli(&cli_path, &task);
            let mut cmd = Command::new(&cli_path);
            cmd.args([
                "--print",
                "--output-format",
                "stream-json",
                "--include-partial-messages",
                &cli_task,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    let pid = child.id();
                    let stdout_output = Arc::new(Mutex::new(String::new()));
                    let stderr_output = Arc::new(Mutex::new(String::new()));

                    sessions.register_process(pid, session_id.clone());
                    sessions.add_progress(
                        &session_id,
                        ProgressEntry {
                            timestamp: chrono::Utc::now().timestamp(),
                            message: format!("Started Claude Code process pid={pid}"),
                            level: LogLevel::Info,
                        },
                    );

                    let stdout_handle = child.stdout.take().map(|stdout| {
                        spawn_output_reader(
                            stdout,
                            "stdout",
                            session_id.clone(),
                            sessions.clone(),
                            stdout_output.clone(),
                            LogLevel::Info,
                        )
                    });
                    let stderr_handle = child.stderr.take().map(|stderr| {
                        spawn_output_reader(
                            stderr,
                            "stderr",
                            session_id.clone(),
                            sessions.clone(),
                            stderr_output.clone(),
                            LogLevel::Warn,
                        )
                    });

                    let wait_result = child.wait();
                    let duration_secs = start.elapsed().as_secs() as i64;

                    if let Some(handle) = stdout_handle {
                        let _ = handle.join();
                    }
                    if let Some(handle) = stderr_handle {
                        let _ = handle.join();
                    }

                    sessions.unregister_process(&pid);

                    let raw_output = combine_output(&stdout_output, &stderr_output);
                    let stderr_text = stderr_output.lock().unwrap().clone();

                    if sessions.is_interrupt_requested(&session_id)
                        || matches!(
                            sessions.get_status(&session_id),
                            Some(SessionStatus::Interrupted)
                        )
                    {
                        sessions.clear_interrupt_requested(&session_id);
                        if !matches!(
                            sessions.get_status(&session_id),
                            Some(SessionStatus::Interrupted)
                        ) {
                            sessions.update_status(&session_id, SessionStatus::Interrupted);
                        }
                        if sessions.get_result(&session_id).is_none() {
                            sessions.set_result(
                                &session_id,
                                ExecutionResult {
                                    summary: String::from("Task interrupted"),
                                    raw_output,
                                    duration_secs,
                                    success: false,
                                    error_message: String::from("Interrupted"),
                                },
                            );
                        }
                        return;
                    }

                    match wait_result {
                        Ok(exit_status) => {
                            let success = exit_status.success();
                            let summary = if success {
                                String::from("Task completed successfully")
                            } else {
                                match exit_status.code() {
                                    Some(code) => format!("Task failed with exit code {code}"),
                                    None => String::from("Task failed"),
                                }
                            };

                            sessions.set_result(
                                &session_id,
                                ExecutionResult {
                                    summary,
                                    raw_output,
                                    duration_secs,
                                    success,
                                    error_message: if success {
                                        String::new()
                                    } else {
                                        stderr_text
                                    },
                                },
                            );
                            sessions.update_status(
                                &session_id,
                                if success {
                                    SessionStatus::Completed
                                } else {
                                    SessionStatus::Failed
                                },
                            );
                        }
                        Err(e) => {
                            sessions.set_result(
                                &session_id,
                                ExecutionResult {
                                    summary: String::from("Task execution failed"),
                                    raw_output,
                                    duration_secs,
                                    success: false,
                                    error_message: e.to_string(),
                                },
                            );
                            sessions.update_status(&session_id, SessionStatus::Failed);
                        }
                    }
                }
                Err(e) => {
                    let duration_secs = start.elapsed().as_secs() as i64;
                    sessions.set_result(
                        &session_id,
                        ExecutionResult {
                            summary: String::from("Failed to spawn process"),
                            raw_output: String::new(),
                            duration_secs,
                            success: false,
                            error_message: e.to_string(),
                        },
                    );
                    sessions.add_progress(
                        &session_id,
                        ProgressEntry {
                            timestamp: chrono::Utc::now().timestamp(),
                            message: format!("Failed to spawn Claude Code process: {e}"),
                            level: LogLevel::Error,
                        },
                    );
                    sessions.update_status(&session_id, SessionStatus::Failed);
                }
            }
        });

        Ok(())
    }

    fn status(&self, session_id: &SessionId) -> Result<Option<crate::session::SessionStatus>> {
        Ok(self.sessions.get_status(session_id))
    }

    fn progress(&self, session_id: &SessionId) -> Result<Option<Vec<ProgressEntry>>> {
        Ok(self.sessions.get_progress(session_id))
    }

    fn interrupt(&self, session_id: &SessionId) -> Result<()> {
        let processes = self.sessions.processes.read();
        let pid_to_kill = processes.iter().find_map(|(pid, sid)| {
            if sid.as_str() == session_id.as_str() {
                Some(*pid)
            } else {
                None
            }
        });
        drop(processes);

        if let Some(pid) = pid_to_kill {
            self.sessions.mark_interrupt_requested(session_id);
            let kill_status = {
                #[cfg(windows)]
                {
                    Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/T", "/F"])
                        .status()
                }
                #[cfg(unix)]
                {
                    Command::new("kill").args(["-9", &pid.to_string()]).status()
                }
            };

            match kill_status {
                Ok(status) if status.success() => {
                    self.sessions.add_progress(
                        session_id,
                        ProgressEntry {
                            timestamp: chrono::Utc::now().timestamp(),
                            message: format!("Interrupted Claude Code process pid={pid}"),
                            level: LogLevel::Warn,
                        },
                    );
                    self.sessions.unregister_process(&pid);
                    self.sessions
                        .update_status(session_id, SessionStatus::Interrupted);
                    Ok(())
                }
                Ok(status) => {
                    self.sessions.clear_interrupt_requested(session_id);
                    Err(Error::Agent(format!(
                        "Failed to interrupt process {pid}, exit={status}"
                    )))
                }
                Err(e) => {
                    self.sessions.clear_interrupt_requested(session_id);
                    Err(Error::Io(e))
                }
            }
        } else {
            Err(Error::Agent(format!(
                "No running process found for session {}",
                session_id
            )))
        }
    }

    fn result(&self, session_id: &SessionId) -> Result<Option<ExecutionResult>> {
        Ok(self.sessions.get_result(session_id))
    }
}

fn spawn_output_reader<R: Read + Send + 'static>(
    reader: R,
    stream_name: &'static str,
    session_id: SessionId,
    sessions: Arc<SessionManager>,
    sink: Arc<Mutex<String>>,
    level: LogLevel,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    {
                        let mut output = sink.lock().unwrap();
                        output.push_str(&line);
                        output.push('\n');
                    }
                    sessions.add_progress(
                        &session_id,
                        ProgressEntry {
                            timestamp: chrono::Utc::now().timestamp(),
                            message: format!("{stream_name}: {line}"),
                            level,
                        },
                    );
                }
                Err(e) => {
                    sessions.add_progress(
                        &session_id,
                        ProgressEntry {
                            timestamp: chrono::Utc::now().timestamp(),
                            message: format!("Failed reading {stream_name}: {e}"),
                            level: LogLevel::Error,
                        },
                    );
                    break;
                }
            }
        }
    })
}

fn combine_output(
    stdout_output: &Arc<Mutex<String>>,
    stderr_output: &Arc<Mutex<String>>,
) -> String {
    let stdout_text = stdout_output.lock().unwrap().clone();
    let stderr_text = stderr_output.lock().unwrap().clone();

    match (stdout_text.trim().is_empty(), stderr_text.trim().is_empty()) {
        (false, true) => stdout_text,
        (true, false) => stderr_text,
        (false, false) => format!("{stdout_text}\n{stderr_text}"),
        (true, true) => String::new(),
    }
}

fn normalize_task_for_cli(cli_path: &str, task: &str) -> String {
    let normalized = task
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if cfg!(windows) && is_batch_entrypoint(cli_path) {
        normalized.replace('\n', " ")
    } else {
        normalized
    }
}

fn is_batch_entrypoint(cli_path: &str) -> bool {
    let lowercase = cli_path.to_ascii_lowercase();
    lowercase.ends_with(".cmd") || lowercase.ends_with(".bat")
}
