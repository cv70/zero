pub mod r#trait;
pub mod context;
pub mod hook;
pub mod loop_config;
pub mod agent_loop;
pub mod token_counter;
pub mod context_manager;

pub use r#trait::{Agent, AgentResponse, ToolCall};
pub use context::{AgentContext, HistoryEntry};
pub use hook::HookedAgent;
pub use loop_config::AgentLoopConfig;
pub use agent_loop::{AgentLoop, DefaultAgentLoop, StreamingAgentLoop};
pub use token_counter::TokenCounter;
pub use context_manager::ContextManager;
