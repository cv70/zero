//! Example 2: Custom Tool
//!
//! This example demonstrates how to create and use a custom Tool implementation.
//! It shows how to:
//! - Implement a custom Tool trait
//! - Define tool metadata (name, description, input schema)
//! - Validate tool input
//! - Execute the tool and handle results
//! - Test the tool
//!
//! Run with: cargo run --example 02-custom-tool

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use zero_core::tool::{Tool, ToolContext, ToolMetadata, ToolOutput};
use zero_core::error::ToolError;

/// A custom Calculator tool that performs basic arithmetic operations
#[derive(Debug, Clone)]
pub struct CalculatorTool;

impl CalculatorTool {
    /// Create a new calculator tool
    pub fn new() -> Self {
        Self
    }
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculator operation types
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Calculator input arguments
#[derive(Debug, Deserialize)]
pub struct CalculatorArgs {
    pub operation: String,
    pub a: f64,
    pub b: f64,
}

impl CalculatorArgs {
    /// Parse and validate the operation
    pub fn parse_operation(&self) -> Result<Operation, ToolError> {
        match self.operation.to_lowercase().as_str() {
            "add" => Ok(Operation::Add),
            "subtract" => Ok(Operation::Subtract),
            "multiply" => Ok(Operation::Multiply),
            "divide" => Ok(Operation::Divide),
            _ => Err(ToolError::InvalidInput(
                "Invalid operation. Use: add, subtract, multiply, or divide".to_string(),
            )),
        }
    }
}

/// Implement the Tool trait for CalculatorTool
#[async_trait]
impl Tool for CalculatorTool {
    /// Return tool metadata (name, description, input schema)
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "calculator".to_string(),
            description: "Perform basic arithmetic operations (add, subtract, multiply, divide)"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "description": "Operation to perform: add, subtract, multiply, divide"
                    },
                    "a": {
                        "type": "number",
                        "description": "First operand"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second operand"
                    }
                },
                "required": ["operation", "a", "b"]
            }),
        }
    }

    /// Validate the tool input before execution
    fn validate_input(&self, input: &str) -> Result<(), ToolError> {
        // Try to deserialize to check format
        serde_json::from_str::<CalculatorArgs>(input)
            .map_err(|e| ToolError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        Ok(())
    }

    /// Execute the calculator operation
    async fn execute(&self, input: &str, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        // Validate input first
        self.validate_input(input)?;

        // Parse arguments
        let args: CalculatorArgs = serde_json::from_str(input)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        // Parse and validate operation
        let operation = args.parse_operation()?;

        // Perform calculation
        let result = match operation {
            Operation::Add => {
                let result = args.a + args.b;
                format!("{} + {} = {}", args.a, args.b, result)
            }
            Operation::Subtract => {
                let result = args.a - args.b;
                format!("{} - {} = {}", args.a, args.b, result)
            }
            Operation::Multiply => {
                let result = args.a * args.b;
                format!("{} * {} = {}", args.a, args.b, result)
            }
            Operation::Divide => {
                if args.b == 0.0 {
                    return Err(ToolError::ExecutionFailed(
                        "Division by zero".to_string(),
                    ));
                }
                let result = args.a / args.b;
                format!("{} / {} = {}", args.a, args.b, result)
            }
        };

        Ok(ToolOutput::text(result))
    }
}

/// Main function demonstrating custom tool usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tool = CalculatorTool::new();
    let ctx = ToolContext::new("example-session".to_string());

    // Print tool metadata
    let metadata = tool.metadata();
    println!("Tool: {}", metadata.name);
    println!("Description: {}", metadata.description);
    println!("Input Schema: {}", serde_json::to_string_pretty(&metadata.input_schema)?);
    println!();

    // Test cases
    let test_cases = vec![
        (r#"{"operation": "add", "a": 10, "b": 5}"#, "10 + 5"),
        (r#"{"operation": "subtract", "a": 10, "b": 3}"#, "10 - 3"),
        (r#"{"operation": "multiply", "a": 6, "b": 7}"#, "6 * 7"),
        (r#"{"operation": "divide", "a": 20, "b": 4}"#, "20 / 4"),
    ];

    println!("Running test cases:");
    println!();

    for (input, description) in &test_cases {
        println!("Test: {}", description);
        match tool.execute(input, &ctx).await {
            Ok(ToolOutput::Text(result)) => {
                println!("  Result: {}", result);
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
            _ => {
                println!("  Unexpected output type");
            }
        }
        println!();
    }

    // Test error case
    println!("Testing error case (division by zero):");
    let error_input = r#"{"operation": "divide", "a": 10, "b": 0}"#;
    match tool.execute(error_input, &ctx).await {
        Ok(_) => println!("  Unexpected: no error"),
        Err(e) => println!("  Expected error: {}", e),
    }

    println!();
    println!("Custom tool example completed!");

    Ok(())
}
