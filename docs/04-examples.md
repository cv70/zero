# Code Examples

Welcome to the Zero Core examples! These examples demonstrate the key concepts of the Agent framework, from basic Agent implementation to multi-Agent coordination. Each example builds upon previous concepts and can be run independently.

## Overview

The examples are organized progressively:

1. **01-simple-agent** - Learn the basics of Agent trait implementation
2. **02-custom-tool** - Implement and use a custom Tool
3. **03-multi-agent** - Coordinate multiple Agents working together

## Example 1: Simple Agent

### Description

This example demonstrates the most basic Agent implementation. You'll learn how to:
- Create an Agent struct that implements the `Agent` trait
- Define basic Agent metadata (name, description, system prompt)
- Implement the async `execute()` method
- Create an agent context and run the agent

### Key Concepts

- **Agent Trait**: The core abstraction for any autonomous agent in the framework
- **Agent Context**: Carries session information and tools available to the agent
- **AgentResponse**: The output type containing the agent's response and metadata

### Running the Example

```bash
cargo run --example 01-simple-agent
```

### Expected Output

```
Agent Name: GreetingAgent
Agent Description: A simple agent that greets users
System Prompt: You are a friendly greeting agent that welcomes users.

Executing agent...
Agent Response:
  Content: Hello! I'm the GreetingAgent. I'm here to help you.
  Tool Calls: 0

Agent execution completed successfully!
```

### Code Walkthrough

The example creates a `GreetingAgent` that implements three key methods:

1. **name()** - Returns the agent's identifier
2. **description()** - Returns a human-readable description
3. **execute()** - The main async function that performs agent logic

```rust
#[async_trait]
impl Agent for GreetingAgent {
    fn name(&self) -> &str {
        "GreetingAgent"
    }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        Ok(AgentResponse {
            content: "Hello! I'm the GreetingAgent. I'm here to help you.".to_string(),
            tool_calls: Vec::new(),
            metadata: std::collections::HashMap::new(),
        })
    }
}
```

### Key Concepts Learned

- Basic Agent trait implementation with async-trait
- Agent lifecycle: creation → context setup → execution
- Returning structured responses with metadata
- Error handling with AgentError

## Example 2: Custom Tool

### Description

This example shows how to create and use a custom Tool implementation. You'll learn to:
- Implement the `Tool` trait for a custom tool
- Define tool metadata with JSON schema for input validation
- Implement input validation before execution
- Handle tool execution with proper error handling
- Test your custom tool

### Key Concepts

- **Tool Trait**: Abstract interface for any tool/capability
- **ToolMetadata**: Name, description, and JSON schema defining tool interface
- **ToolContext**: Execution context containing session and environment info
- **ToolOutput**: Result type supporting multiple output formats
- **Input Validation**: Ensuring tool receives valid input

### Running the Example

```bash
cargo run --example 02-custom-tool
```

### Expected Output

```
Tool: calculator
Description: Perform basic arithmetic operations (add, subtract, multiply, divide)
Input Schema: { ... }

Running test cases:

Test: 10 + 5
  Result: 10 + 5 = 15

Test: 10 - 3
  Result: 10 - 3 = 7

Test: 6 * 7
  Result: 6 * 7 = 42

Test: 20 / 4
  Result: 20 / 4 = 5

Testing error case (division by zero):
  Expected error: Execution failed: Division by zero

Custom tool example completed!
```

### Code Walkthrough

The example implements a `CalculatorTool` with the following structure:

```rust
#[async_trait]
impl Tool for CalculatorTool {
    fn metadata(&self) -> ToolMetadata {
        // Define tool interface
        ToolMetadata {
            name: "calculator".to_string(),
            description: "Perform basic arithmetic operations".to_string(),
            input_schema: json!({ /* JSON Schema */ }),
        }
    }

    fn validate_input(&self, input: &str) -> Result<(), ToolError> {
        // Optional validation before execution
        serde_json::from_str::<CalculatorArgs>(input)
            .map_err(|e| ToolError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        Ok(())
    }

    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        // Parse, validate, execute
        let args: CalculatorArgs = serde_json::from_str(input)?;

        // Perform the operation
        let result = match args.operation.as_str() {
            "add" => args.a + args.b,
            // ... other operations
        };

        Ok(ToolOutput::text(result.to_string()))
    }
}
```

### Key Concepts Learned

- Tool trait implementation with full lifecycle
- JSON schema definition for tool interfaces
- Input validation and error handling
- Structured serialization with serde
- Testing tools independently

## Example 3: Multi-Agent Coordination

### Description

This example demonstrates how to create and coordinate multiple Agents working together. You'll learn to:
- Create multiple agents with different responsibilities
- Build an Agent coordinator to manage multiple agents
- Execute agents sequentially with shared context
- Collect and summarize results from multiple agents
- Design multi-agent workflows

### Key Concepts

- **Agent Diversity**: Different agents specialized for different tasks
- **Coordination**: Managing execution order and information flow
- **Shared Context**: Passing information between agents
- **Result Aggregation**: Collecting and summarizing agent outputs

### Running the Example

```bash
cargo run --example 03-multi-agent
```

### Expected Output

```
=== Multi-Agent Coordination Example ===

Registered agents:
  - ResearchAgent: Gathers and researches information on given topics
  - AnalysisAgent: Analyzes information and draws conclusions
  - ReportAgent: Generates comprehensive reports from analyzed data

Executing agents in coordination...

Executing agent: ResearchAgent
  Result: Research complete on topic...
Executing agent: AnalysisAgent
  Result: Analysis complete...
Executing agent: ReportAgent
  Result: Executive Summary...

=== Execution Summary ===
Total agents executed: 3

Agent 1 Response:
  Content: Research complete...
  Tool Calls: 0
  Metadata:
    research_items: 3
    confidence: high
...
```

### Code Walkthrough

The example creates three specialized agents:

1. **ResearchAgent** - Gathers information
2. **AnalysisAgent** - Analyzes findings
3. **ReportAgent** - Generates final report

And an `AgentCoordinator` to manage them:

```rust
pub struct AgentCoordinator {
    agents: Vec<(String, Box<dyn Agent>)>,
}

impl AgentCoordinator {
    pub async fn execute_all(&self, context: &AgentContext)
        -> Result<Vec<AgentResponse>, AgentError> {
        let mut responses = Vec::new();

        for (name, agent) in &self.agents {
            let response = agent.execute(context).await?;
            responses.push(response);
        }

        Ok(responses)
    }
}
```

### Key Concepts Learned

- Creating multiple agents with different personalities
- Agent composition and coordination patterns
- Sequential vs. parallel execution strategies
- Result aggregation and summary
- Designing reusable coordinator components

## Running All Examples

To compile all examples without running:

```bash
cargo build --examples
```

To run a specific example:

```bash
cargo run --example 01-simple-agent
cargo run --example 02-custom-tool
cargo run --example 03-multi-agent
```

## What's Next?

After exploring these examples, you're ready to:

1. **Read the API Documentation**: Check `docs/01-getting-started.md` for detailed API information
2. **Explore Advanced Features**: Review `docs/02-core-concepts.md` for advanced agent execution patterns
3. **Implement Your Own Agents**: Use these examples as templates for your custom agents and tools
4. **Build Multi-Agent Systems**: Combine multiple agents to solve complex problems

## Tips for Extending Examples

### Creating Your Own Agent

```rust
#[derive(Debug, Clone)]
pub struct MyCustomAgent;

#[async_trait]
impl Agent for MyCustomAgent {
    fn name(&self) -> &str { "MyAgent" }

    async fn execute(&self, context: &AgentContext) -> Result<AgentResponse, AgentError> {
        // Your implementation
        Ok(AgentResponse {
            content: "Response".to_string(),
            tool_calls: Vec::new(),
            metadata: HashMap::new(),
        })
    }
}
```

### Creating Your Own Tool

```rust
#[derive(Debug, Clone)]
pub struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "my_tool".to_string(),
            description: "Description".to_string(),
            input_schema: json!({ /* schema */ }),
        }
    }

    async fn execute(&self, input: &str, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        // Your implementation
        Ok(ToolOutput::text("result".to_string()))
    }
}
```

## Troubleshooting

### Examples Don't Compile
- Ensure you're using Rust 1.85 or later (check with `rustc --version`)
- Run `cargo clean` and rebuild if experiencing build issues
- Check that you're in the project root directory

### Examples Panic on Execution
- Check error messages for required setup (environment variables, file paths)
- Ensure all workspace dependencies are properly configured
- Review the example code comments for context requirements

### Tool Execution Fails
- Verify input JSON matches the schema defined in `metadata()`
- Check error messages for validation failures
- Enable logging with `RUST_LOG=debug` for more details

## Additional Resources

- **Getting Started**: See `docs/01-getting-started.md`
- **Core Concepts**: See `docs/02-core-concepts.md`
- **Trait Architecture**: See `docs/03-trait-architecture.md`
- **Project Architecture**: See `ARCHITECTURE.md`
- **API Documentation**: Run `cargo doc --open` to browse generated docs

## Contributing Examples

If you'd like to contribute new examples:

1. Create a new file following the naming convention: `NN-description.rs`
2. Add inline documentation explaining the example
3. Ensure the example compiles and runs successfully
4. Document the example in this file
5. Submit a pull request

Happy coding! Explore these examples to understand the Zero framework and build powerful multi-agent systems.
