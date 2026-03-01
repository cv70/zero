pub mod r#trait;
pub mod metadata;
pub mod registry;
pub mod dispatcher;
pub mod builtins;

pub use r#trait::{Tool, ToolOutput, ToolMetadata, ToolContext};
pub use metadata::ToolDefinition;
pub use registry::ToolRegistry;
pub use dispatcher::{ToolDispatcher, SimpleToolDispatcher, RegistryToolDispatcher};
pub use builtins::{BashTool, ReadFileTool, WriteFileTool, EditFileTool};

/// Tool call request
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}
