//! Example 1: Simple Agent
//!
//! This example demonstrates the most basic Agent implementation.
//! It shows how to:
//! - Create a simple Agent struct
//! - Implement the Agent trait
//! - Execute the agent with a context
//!
//! Run with: cargo run --example 01-simple-agent

use async_trait::async_trait;
use zero_core::agent::{Agent, AgentContext, AgentResponse};
use zero_core::error::AgentError;

/// A simple greeting agent that always returns a fixed greeting message
#[derive(Debug, Clone)]
pub struct GreetingAgent;

impl GreetingAgent {
    /// Create a new greeting agent
    pub fn new() -> Self {
        Self
    }
}

impl Default for GreetingAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement the Agent trait for GreetingAgent
#[async_trait]
impl Agent for GreetingAgent {
    /// Return the agent's name
    fn name(&self) -> &str {
        "GreetingAgent"
    }

    /// Return the agent's system prompt
    fn system_prompt(&self) -> &str {
        "You are a friendly greeting agent that welcomes users."
    }

    /// Return the agent's description
    fn description(&self) -> &str {
        "A simple agent that greets users"
    }

    /// Execute the agent - the main async function
    async fn execute(&self, _context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // For this simple agent, we just return a static greeting
        Ok(AgentResponse {
            content: "Hello! I'm the GreetingAgent. I'm here to help you.".to_string(),
            tool_calls: Vec::new(),
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// Main function demonstrating agent execution
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an agent instance
    let agent = GreetingAgent::new();

    // Create an agent context with a unique session ID
    let context = AgentContext::new("session-001".to_string());

    // Print basic agent information
    println!("Agent Name: {}", agent.name());
    println!("Agent Description: {}", agent.description());
    println!("System Prompt: {}", agent.system_prompt());
    println!();

    // Execute the agent
    println!("Executing agent...");
    match agent.execute(&context).await {
        Ok(response) => {
            println!("Agent Response:");
            println!("  Content: {}", response.content);
            println!("  Tool Calls: {}", response.tool_calls.len());
        }
        Err(e) => {
            eprintln!("Error executing agent: {}", e);
            return Err(e.into());
        }
    }

    println!();
    println!("Agent execution completed successfully!");

    Ok(())
}
