//! Example 3: Multi-Agent Coordination
//!
//! This example demonstrates coordinating multiple agents to solve a complex task.
//! It shows how to:
//! - Create multiple agents with different responsibilities
//! - Coordinate agents to work together
//! - Pass context and information between agents
//! - Handle parallel agent execution
//!
//! Run with: cargo run --example 03-multi-agent

use async_trait::async_trait;
use std::collections::HashMap;
use zero_core::agent::{Agent, AgentContext, AgentResponse};
use zero_core::error::AgentError;

/// A Research Agent that gathers information
#[derive(Debug, Clone)]
pub struct ResearchAgent;

impl ResearchAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResearchAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for ResearchAgent {
    fn name(&self) -> &str {
        "ResearchAgent"
    }

    fn description(&self) -> &str {
        "Gathers and researches information on given topics"
    }

    fn system_prompt(&self) -> &str {
        "You are a research expert. Your role is to gather comprehensive information on topics."
    }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // Simulate research findings
        let findings = vec![
            "Initial research completed",
            "Data sources identified",
            "Key insights collected",
        ];

        let mut metadata = HashMap::new();
        metadata.insert("research_items".to_string(), "3".to_string());
        metadata.insert("confidence".to_string(), "high".to_string());

        Ok(AgentResponse {
            content: format!(
                "Research complete on topic. Found {}. Session: {}",
                findings.join(", "),
                context.session_id
            ),
            tool_calls: Vec::new(),
            metadata,
        })
    }
}

/// An Analysis Agent that analyzes information
#[derive(Debug, Clone)]
pub struct AnalysisAgent;

impl AnalysisAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalysisAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for AnalysisAgent {
    fn name(&self) -> &str {
        "AnalysisAgent"
    }

    fn description(&self) -> &str {
        "Analyzes information and draws conclusions"
    }

    fn system_prompt(&self) -> &str {
        "You are an analyst. Your role is to analyze information and identify patterns."
    }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // Simulate analysis
        let analysis = vec![
            "Pattern analysis completed",
            "Trends identified",
            "Conclusions derived",
        ];

        let mut metadata = HashMap::new();
        metadata.insert("patterns_found".to_string(), "5".to_string());
        metadata.insert("trend_strength".to_string(), "strong".to_string());

        Ok(AgentResponse {
            content: format!(
                "Analysis complete. Results: {}. Session: {}",
                analysis.join(", "),
                context.session_id
            ),
            tool_calls: Vec::new(),
            metadata,
        })
    }
}

/// A Report Agent that generates reports
#[derive(Debug, Clone)]
pub struct ReportAgent;

impl ReportAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReportAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for ReportAgent {
    fn name(&self) -> &str {
        "ReportAgent"
    }

    fn description(&self) -> &str {
        "Generates comprehensive reports from analyzed data"
    }

    fn system_prompt(&self) -> &str {
        "You are a report writer. Your role is to create clear, actionable reports."
    }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // Simulate report generation
        let mut metadata = HashMap::new();
        metadata.insert("report_format".to_string(), "comprehensive".to_string());
        metadata.insert("sections".to_string(), "5".to_string());

        Ok(AgentResponse {
            content: format!(
                "Executive Summary: Analysis and research combined for comprehensive report. Session: {}",
                context.session_id
            ),
            tool_calls: Vec::new(),
            metadata,
        })
    }
}

/// Coordinator for managing multiple agents
pub struct AgentCoordinator {
    agents: Vec<(String, Box<dyn Agent>)>,
}

impl AgentCoordinator {
    /// Create a new agent coordinator
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
        }
    }

    /// Register an agent with the coordinator
    pub fn register_agent(&mut self, name: String, agent: Box<dyn Agent>) {
        self.agents.push((name, agent));
    }

    /// Execute all agents sequentially
    pub async fn execute_all(&self, context: &AgentContext) -> Result<Vec<AgentResponse>, AgentError> {
        let mut responses = Vec::new();

        for (name, agent) in &self.agents {
            println!("Executing agent: {}", name);
            match agent.execute(context).await {
                Ok(response) => {
                    println!("  Result: {}", response.content);
                    responses.push(response);
                }
                Err(e) => {
                    eprintln!("  Error: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(responses)
    }

    /// Get all registered agents
    pub fn agents(&self) -> &[(String, Box<dyn Agent>)] {
        &self.agents
    }
}

impl Default for AgentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Main function demonstrating multi-agent coordination
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Agent Coordination Example ===");
    println!();

    // Create individual agents
    let research_agent: Box<dyn Agent> = Box::new(ResearchAgent::new());
    let analysis_agent: Box<dyn Agent> = Box::new(AnalysisAgent::new());
    let report_agent: Box<dyn Agent> = Box::new(ReportAgent::new());

    // Create coordinator and register agents
    let mut coordinator = AgentCoordinator::new();
    coordinator.register_agent("ResearchAgent".to_string(), research_agent);
    coordinator.register_agent("AnalysisAgent".to_string(), analysis_agent);
    coordinator.register_agent("ReportAgent".to_string(), report_agent);

    // Create context for the coordination
    let context = AgentContext::new("multi-agent-session-001".to_string());

    // Print registered agents
    println!("Registered agents:");
    for (name, agent) in coordinator.agents() {
        println!("  - {}: {}", name, agent.description());
    }
    println!();

    // Execute all agents in sequence
    println!("Executing agents in coordination...");
    println!();

    match coordinator.execute_all(&context).await {
        Ok(responses) => {
            println!();
            println!("=== Execution Summary ===");
            println!("Total agents executed: {}", responses.len());
            println!();

            for (idx, response) in responses.iter().enumerate() {
                println!("Agent {} Response:", idx + 1);
                println!("  Content: {}", response.content);
                println!("  Tool Calls: {}", response.tool_calls.len());
                if !response.metadata.is_empty() {
                    println!("  Metadata:");
                    for (key, value) in &response.metadata {
                        println!("    {}: {}", key, value);
                    }
                }
                println!();
            }

            println!("Multi-agent coordination completed successfully!");
        }
        Err(e) => {
            eprintln!("Error during coordination: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
