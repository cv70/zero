use crate::tool::Tool;

/// Agent 执行上下文
pub struct AgentContext {
    pub session_id: String,
    pub tools: Vec<Box<dyn Tool>>,
    pub history: Vec<HistoryEntry>,
}

impl AgentContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            tools: Vec::new(),
            history: Vec::new(),
        }
    }
}

/// 历史条目
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub role: String,
    pub content: String,
}
