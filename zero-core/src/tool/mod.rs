pub mod builtins;
pub mod dispatcher;
pub mod metadata;
pub mod registry;
pub mod r#trait;

pub use builtins::{BashTool, EditFileTool, ReadFileTool, WriteFileTool};
pub use dispatcher::{RegistryToolDispatcher, SimpleToolDispatcher, ToolDispatcher};
pub use metadata::ToolDefinition;
pub use registry::ToolRegistry;
pub use r#trait::{Tool, ToolContext, ToolMetadata, ToolOutput};

/// Tool call request
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}
