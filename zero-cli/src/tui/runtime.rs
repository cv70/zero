#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEvent {
    TokenDelta { session_id: usize, text: String },
    ToolEvent { session_id: usize, name: String },
    Done { session_id: usize },
    Error { session_id: usize, message: String },
}
