use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

use zero_core::agent::{AgentLoop, AgentLoopConfig, DefaultAgentLoop, StreamingAgentLoop};
use zero_core::message::Message;
use zero_core::provider::{
    AnthropicLoopProvider, LoopProvider, OllamaLoopProvider, OpenAILoopProvider, StreamEvent,
    StreamingLoopProvider,
};
use zero_core::tool::{
    BashTool, EditFileTool, ReadFileTool, RegistryToolDispatcher, Tool, ToolDispatcher,
    ToolRegistry, WriteFileTool,
};

/// Zero Agent CLI - an interactive AI coding assistant
#[derive(Parser, Debug)]
#[command(name = "zero-cli", about = "Zero Agent interactive CLI")]
struct Cli {
    /// Provider to use: anthropic, openai, ollama
    #[arg(short, long, default_value = "anthropic")]
    provider: String,

    /// Model name (provider-specific default if omitted)
    #[arg(short, long)]
    model: Option<String>,

    /// API key (falls back to ANTHROPIC_API_KEY / OPENAI_API_KEY env vars)
    #[arg(short = 'k', long = "api-key")]
    api_key: Option<String>,

    /// System prompt
    #[arg(short, long)]
    system: Option<String>,

    /// Custom API endpoint (for Ollama, default: http://localhost:11434)
    #[arg(long)]
    endpoint: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable streaming output (Anthropic only)
    #[arg(long)]
    stream: bool,

    /// Max context tokens before compaction (0 = disabled)
    #[arg(long, default_value = "180000")]
    max_context_tokens: usize,

    /// Prompt for single-shot mode (if provided, execute and exit)
    prompt: Option<String>,

    /// Enable shadow mode for compatibility rollout
    #[arg(long, default_value_t = false)]
    shadow_mode: bool,
}

fn setup_tracing(verbose: bool) {
    if verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .with_target(false)
            .init();
    }
}

/// Resolve the API key from CLI flag or environment variable.
fn resolve_api_key(cli_key: &Option<String>, provider: &str) -> Result<String> {
    if let Some(key) = cli_key {
        return Ok(key.clone());
    }

    let env_var = match provider {
        "anthropic" => "ANTHROPIC_API_KEY",
        "openai" => "OPENAI_API_KEY",
        _ => return Err(anyhow!("No API key env var for provider '{}'", provider)),
    };

    std::env::var(env_var).with_context(|| {
        format!(
            "API key not provided via --api-key and {} is not set",
            env_var
        )
    })
}

/// Build tool definitions in the format expected by the given provider.
fn build_tool_definitions(tools: &[Box<dyn Tool>], provider_name: &str) -> Vec<serde_json::Value> {
    tools
        .iter()
        .map(|tool| {
            let meta = tool.metadata();
            match provider_name {
                "anthropic" => {
                    serde_json::json!({
                        "name": meta.name,
                        "description": meta.description,
                        "input_schema": meta.input_schema,
                    })
                }
                "openai" | "ollama" => {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": meta.name,
                            "description": meta.description,
                            "parameters": meta.input_schema,
                        }
                    })
                }
                _ => {
                    serde_json::json!({
                        "name": meta.name,
                        "description": meta.description,
                        "input_schema": meta.input_schema,
                    })
                }
            }
        })
        .collect()
}

/// Whether the provider supports streaming
enum ProviderKind {
    Standard(Arc<dyn LoopProvider>),
    Streaming(Arc<dyn StreamingLoopProvider>),
}

/// Create the appropriate LoopProvider based on CLI arguments.
fn create_provider(cli: &Cli, tool_defs: Vec<serde_json::Value>) -> Result<(ProviderKind, String)> {
    match cli.provider.as_str() {
        "anthropic" => {
            let api_key = resolve_api_key(&cli.api_key, "anthropic")?;
            let mut provider = AnthropicLoopProvider::new(api_key).with_tools(tool_defs);
            if let Some(ref model) = cli.model {
                provider = provider.with_model(model);
            }
            if let Some(ref system) = cli.system {
                provider = provider.with_system_prompt(system);
            }
            let model_name = cli
                .model
                .clone()
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

            if cli.stream {
                Ok((
                    ProviderKind::Streaming(Arc::new(provider) as Arc<dyn StreamingLoopProvider>),
                    model_name,
                ))
            } else {
                Ok((
                    ProviderKind::Standard(Arc::new(provider) as Arc<dyn LoopProvider>),
                    model_name,
                ))
            }
        }
        "openai" => {
            let api_key = resolve_api_key(&cli.api_key, "openai")?;
            let mut provider = OpenAILoopProvider::new(api_key).with_tools(tool_defs);
            if let Some(ref model) = cli.model {
                provider = provider.with_model(model);
            }
            if let Some(ref system) = cli.system {
                provider = provider.with_system_prompt(system);
            }
            let model_name = cli.model.clone().unwrap_or_else(|| "gpt-4o".to_string());
            Ok((
                ProviderKind::Standard(Arc::new(provider) as Arc<dyn LoopProvider>),
                model_name,
            ))
        }
        "ollama" => {
            let mut provider = OllamaLoopProvider::new().with_tools(tool_defs);
            if let Some(ref model) = cli.model {
                provider = provider.with_model(model);
            }
            if let Some(ref endpoint) = cli.endpoint {
                provider = provider.with_endpoint(endpoint);
            }
            if let Some(ref system) = cli.system {
                provider = provider.with_system_prompt(system);
            }
            let model_name = cli.model.clone().unwrap_or_else(|| "llama3.2".to_string());
            Ok((
                ProviderKind::Standard(Arc::new(provider) as Arc<dyn LoopProvider>),
                model_name,
            ))
        }
        other => Err(anyhow!(
            "Unknown provider '{}'. Supported: anthropic, openai, ollama",
            other
        )),
    }
}

/// Print the final text response from the agent.
fn print_response(response: &str) {
    if !response.is_empty() {
        println!("{}", response);
    }
}

/// Run the agent in single-shot mode (non-streaming).
async fn run_single_shot(
    agent_loop: &DefaultAgentLoop,
    config: &AgentLoopConfig,
    prompt: &str,
) -> Result<()> {
    let mut messages = vec![Message::user(prompt)];
    let response = agent_loop
        .execute(&mut messages, config)
        .await
        .map_err(|e| anyhow!("Agent error: {}", e))?;
    print_response(&response);
    Ok(())
}

/// Run the agent in single-shot mode (streaming).
async fn run_single_shot_streaming(
    agent_loop: &StreamingAgentLoop,
    config: &AgentLoopConfig,
    prompt: &str,
) -> Result<()> {
    let mut messages = vec![Message::user(prompt)];
    let _response = agent_loop
        .execute_streaming(&mut messages, config, |event| {
            handle_stream_event(&event);
        })
        .await
        .map_err(|e| anyhow!("Agent error: {}", e))?;
    println!();
    Ok(())
}

/// Handle a single streaming event for display.
fn handle_stream_event(event: &StreamEvent) {
    match event {
        StreamEvent::TextDelta(text) => {
            print!("{}", text);
            let _ = io::stdout().flush();
        }
        StreamEvent::ToolUseStart { name, .. } => {
            println!("\n[Tool: {}]", name);
        }
        StreamEvent::ContentBlockStop => {}
        StreamEvent::ToolUseInputDelta(_) => {}
        StreamEvent::MessageStop { .. } => {}
    }
}

/// Run the interactive REPL loop (non-streaming).
async fn run_repl(
    agent_loop: &DefaultAgentLoop,
    config: &AgentLoopConfig,
    provider_name: &str,
    model_name: &str,
) -> Result<()> {
    println!(
        "Zero Agent (provider: {}, model: {})",
        provider_name, model_name
    );
    println!("Type 'exit' or 'quit' to exit, Ctrl-C to cancel.");
    println!();

    let stdin = io::stdin();
    let mut messages: Vec<Message> = Vec::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line)?;

        if bytes_read == 0 {
            println!();
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        messages.push(Message::user(input));

        println!("[Agent thinking...]");
        match agent_loop.execute(&mut messages, config).await {
            Ok(response) => {
                print_response(&response);
                println!();
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                println!();
            }
        }
    }

    Ok(())
}

/// Run the interactive REPL loop (streaming).
async fn run_repl_streaming(
    agent_loop: &StreamingAgentLoop,
    config: &AgentLoopConfig,
    provider_name: &str,
    model_name: &str,
) -> Result<()> {
    println!(
        "Zero Agent (provider: {}, model: {}, streaming: on)",
        provider_name, model_name
    );
    println!("Type 'exit' or 'quit' to exit, Ctrl-C to cancel.");
    println!();

    let stdin = io::stdin();
    let mut messages: Vec<Message> = Vec::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line)?;

        if bytes_read == 0 {
            println!();
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        messages.push(Message::user(input));

        match agent_loop
            .execute_streaming(&mut messages, config, |event| {
                handle_stream_event(&event);
            })
            .await
        {
            Ok(_) => {
                println!();
                println!();
            }
            Err(e) => {
                eprintln!("\nError: {}", e);
                println!();
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.shadow_mode {
        eprintln!("[compat] shadow mode enabled");
    }

    setup_tracing(cli.verbose);

    // Create built-in tools for schema extraction
    let builtin_tools: Vec<Box<dyn Tool>> = vec![
        Box::new(BashTool::new()),
        Box::new(ReadFileTool),
        Box::new(WriteFileTool),
        Box::new(EditFileTool),
    ];

    // Build tool definitions in provider-specific format
    let tool_defs = build_tool_definitions(&builtin_tools, &cli.provider);

    // Create provider
    let (provider_kind, model_name) = create_provider(&cli, tool_defs)?;
    let provider_name = cli.provider.clone();

    // Register tools in the registry
    let registry = Arc::new(ToolRegistry::new());
    registry.register(Box::new(BashTool::new())).await;
    registry.register(Box::new(ReadFileTool)).await;
    registry.register(Box::new(WriteFileTool)).await;
    registry.register(Box::new(EditFileTool)).await;

    let dispatcher = Arc::new(RegistryToolDispatcher::new(registry));

    // Create loop config
    let config = AgentLoopConfig::default()
        .with_verbose_logging(cli.verbose)
        .with_max_context_tokens(cli.max_context_tokens);

    // Handle Ctrl-C gracefully
    let ctrl_c = tokio::spawn(async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C handler");
    });

    match provider_kind {
        ProviderKind::Standard(provider) => {
            let agent_loop = DefaultAgentLoop::new(provider, dispatcher as Arc<dyn ToolDispatcher>);

            if let Some(ref prompt) = cli.prompt {
                tokio::select! {
                    result = run_single_shot(&agent_loop, &config, prompt) => {
                        result?;
                    }
                    _ = ctrl_c => {
                        eprintln!("\nInterrupted.");
                    }
                }
            } else {
                tokio::select! {
                    result = run_repl(&agent_loop, &config, &provider_name, &model_name) => {
                        result?;
                    }
                    _ = ctrl_c => {
                        eprintln!("\nInterrupted.");
                    }
                }
            }
        }
        ProviderKind::Streaming(provider) => {
            let agent_loop =
                StreamingAgentLoop::new(provider, dispatcher as Arc<dyn ToolDispatcher>);

            if let Some(ref prompt) = cli.prompt {
                tokio::select! {
                    result = run_single_shot_streaming(&agent_loop, &config, prompt) => {
                        result?;
                    }
                    _ = ctrl_c => {
                        eprintln!("\nInterrupted.");
                    }
                }
            } else {
                tokio::select! {
                    result = run_repl_streaming(&agent_loop, &config, &provider_name, &model_name) => {
                        result?;
                    }
                    _ = ctrl_c => {
                        eprintln!("\nInterrupted.");
                    }
                }
            }
        }
    }

    Ok(())
}
