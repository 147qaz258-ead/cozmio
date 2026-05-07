use crate::error::Result;
use crate::session::{ExecutionResult, ProgressEntry, SessionId};

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait ExecutionAgent: Send + Sync {
    fn name(&self) -> &str;
    fn dispatch(&self, session_id: SessionId, task: String) -> Result<()>;
    fn status(&self, session_id: &SessionId) -> Result<Option<crate::session::SessionStatus>>;
    fn progress(&self, session_id: &SessionId) -> Result<Option<Vec<ProgressEntry>>>;
    fn interrupt(&self, session_id: &SessionId) -> Result<()>;
    fn result(&self, session_id: &SessionId) -> Result<Option<ExecutionResult>>;
}

pub struct AgentRegistry {
    agents: RwLock<HashMap<String, Arc<dyn ExecutionAgent>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        AgentRegistry {
            agents: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, agent: Arc<dyn ExecutionAgent>) {
        let name = agent.name().to_string();
        self.agents.write().insert(name, agent);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn ExecutionAgent>> {
        self.agents.read().get(name).cloned()
    }

    pub fn list(&self) -> Vec<String> {
        self.agents.read().keys().cloned().collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
