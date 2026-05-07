use crate::config::Config;
use crate::logging::{ActionLogger, FactualActionRecord, FactualEventType, SystemRoute};
use crate::model_client::{InterventionMode, ModelOutput};

/// Result of routing an output through the executor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionResult {
    /// Action was notified (suggest level)
    Notified,
    /// User confirmed the action (request level, user approved)
    Confirmed,
    /// Action was executed (execute level, either auto or confirmed)
    Executed,
    /// Action was skipped (user declined or auto disabled)
    Skipped,
}

impl std::fmt::Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionResult::Notified => write!(f, "notified"),
            ExecutionResult::Confirmed => write!(f, "confirmed"),
            ExecutionResult::Executed => write!(f, "executed"),
            ExecutionResult::Skipped => write!(f, "skipped"),
        }
    }
}

/// Executor handles routing of model outputs to appropriate actions
pub struct Executor {
    config: Config,
    logger: ActionLogger,
}

impl Executor {
    /// Create a new Executor with the given config and logger
    pub fn new(config: Config, logger: ActionLogger) -> Self {
        Self { config, logger }
    }

    /// Route a model output to the appropriate action based on its level
    ///
    /// Returns the result of routing, or an error string if routing failed.
    /// The actual notification/dialog execution should be handled by the caller
    /// based on the returned ExecutionResult.
    pub fn route(
        &self,
        output: &ModelOutput,
        window_title: &str,
    ) -> Result<ExecutionResult, String> {
        // Create FactualActionRecord (not legacy ActionRecord)
        let record = FactualActionRecord {
            timestamp: chrono::Utc::now().timestamp(),
            trace_id: None,
            session_id: None,
            window_title: window_title.to_string(),
            event_type: FactualEventType::ModelOutput,
            system_route: match output.mode {
                InterventionMode::Continue => SystemRoute::Continue,
                InterventionMode::Abstain => SystemRoute::Abstain,
            },
            original_judgment: output.mode.to_string(), // Model's judgment preserved
            execution_result_str: String::new(),        // Will be set after execution
            raw_model_text: Some(output.reason.clone()), // Model original text preserved
            model_name: None,
            captured_at: None,
            call_started_at: None,
            call_duration_ms: None,
            execution_result: None,
            error_text: None,
            user_feedback: None,
        };

        let result = match output.mode {
            InterventionMode::Continue => self.handle_continue(output, &record),
            InterventionMode::Abstain => self.handle_abstain(output, &record),
        };

        if let Ok(ref exec_result) = result {
            let mut updated_record = record;
            updated_record.execution_result_str = exec_result.to_string();
            updated_record.system_route = match exec_result {
                ExecutionResult::Notified => SystemRoute::Unknown,
                ExecutionResult::Confirmed => SystemRoute::Confirmed,
                ExecutionResult::Executed => SystemRoute::AutoExecuted,
                ExecutionResult::Skipped => SystemRoute::Declined,
            };
            // Use log_factual when available, fallback to log_legacy
            if let Err(e) = self.logger.log_factual(updated_record) {
                eprintln!("Failed to log action: {}", e);
            }
        }

        result
    }

    fn handle_continue(
        &self,
        output: &ModelOutput,
        _record: &FactualActionRecord,
    ) -> Result<ExecutionResult, String> {
        log::info!("CONTINUE: reason: {}", output.reason,);
        Ok(ExecutionResult::Confirmed)
    }

    fn handle_abstain(
        &self,
        output: &ModelOutput,
        _record: &FactualActionRecord,
    ) -> Result<ExecutionResult, String> {
        log::info!("ABSTAIN: reason: {}", output.reason,);
        Ok(ExecutionResult::Skipped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_client::InterventionMode;
    use serial_test::serial;
    use std::fs;

    fn create_test_config() -> Config {
        Config {
            ollama_url: "http://localhost:11434".to_string(),
            model_name: "llava".to_string(),
            poll_interval_secs: 3,
            window_change_detection: true,
            execute_auto: true,
            request_use_native_dialog: true,
            execute_delay_secs: 1,
            memory_flywheel_enabled: true,
            consolidation_model_name: None,
            last_check_at: None,
            update_channel: "stable".to_string(),
            memory_maintenance_mode: "local".to_string(),
        }
    }

    fn create_test_logger() -> ActionLogger {
        let temp_dir = std::env::temp_dir().join("cozmio_executor_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        ActionLogger::with_path(temp_dir.join("action_log.jsonl"))
    }

    fn create_test_output(mode: InterventionMode) -> ModelOutput {
        ModelOutput {
            mode,
            reason: "Test reason".to_string(),
            user_how: None,
        }
    }

    #[test]
    #[serial]
    fn test_execution_result_display() {
        assert_eq!(ExecutionResult::Notified.to_string(), "notified");
        assert_eq!(ExecutionResult::Confirmed.to_string(), "confirmed");
        assert_eq!(ExecutionResult::Executed.to_string(), "executed");
        assert_eq!(ExecutionResult::Skipped.to_string(), "skipped");
    }

    #[test]
    #[serial]
    fn test_route_continue_returns_confirmed() {
        let config = create_test_config();
        let logger = create_test_logger();
        let executor = Executor::new(config, logger);

        let output = create_test_output(InterventionMode::Continue);
        let result = executor.route(&output, "Test Window");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExecutionResult::Confirmed);
    }

    #[test]
    #[serial]
    fn test_route_abstain_returns_skipped() {
        let config = create_test_config();
        let logger = create_test_logger();
        let executor = Executor::new(config, logger);

        let output = create_test_output(InterventionMode::Abstain);
        let result = executor.route(&output, "Test Window");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExecutionResult::Skipped);
    }

    #[test]
    #[serial]
    fn test_route_continue_logs_action() {
        let config = create_test_config();
        let logger = create_test_logger();
        let executor = Executor::new(config, logger.clone());

        let output = create_test_output(InterventionMode::Continue);
        executor.route(&output, "Test Window").unwrap();

        let recent = logger.get_recent(1).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].window_title, "Test Window");
        assert_eq!(recent[0].judgment, "CONTINUE");
        assert_eq!(recent[0].system_action, "confirmed");
    }

    #[test]
    #[serial]
    fn test_route_abstain_logs_action() {
        let config = create_test_config();
        let logger = create_test_logger();
        let executor = Executor::new(config, logger.clone());

        let output = create_test_output(InterventionMode::Abstain);
        executor.route(&output, "Test Window").unwrap();

        let recent = logger.get_recent(1).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].window_title, "Test Window");
        assert_eq!(recent[0].judgment, "ABSTAIN");
        assert_eq!(recent[0].system_action, "skipped");
    }

    #[test]
    #[serial]
    fn test_executor_logs_correct_judgment_for_continue() {
        let config = create_test_config();
        let logger = create_test_logger();
        let executor = Executor::new(config, logger.clone());

        let output = ModelOutput {
            mode: InterventionMode::Continue,
            reason: "Evidence supports continuation".to_string(),
            user_how: None,
        };
        executor.route(&output, "Test Window").unwrap();
        let recent = logger.get_recent(1).unwrap();
        assert_eq!(recent[0].judgment, "CONTINUE");
        assert_eq!(recent[0].system_action, "confirmed");
    }

    #[test]
    #[serial]
    fn test_executor_logs_correct_judgment_for_abstain() {
        let config = create_test_config();
        let logger = create_test_logger();
        let executor = Executor::new(config, logger.clone());

        let output = ModelOutput {
            mode: InterventionMode::Abstain,
            reason: "Evidence insufficient".to_string(),
            user_how: None,
        };
        executor.route(&output, "Test Window").unwrap();
        let recent = logger.get_recent(1).unwrap();
        assert_eq!(recent[0].judgment, "ABSTAIN");
        assert_eq!(recent[0].system_action, "skipped");
    }
}
