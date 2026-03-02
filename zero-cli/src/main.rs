use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

use zero_core::ZeroInit;
use zero_core::agent::{AgentLoop, AgentLoopConfig, DefaultAgentLoop, StreamingAgentLoop};
use zero_core::config::Config;
use zero_core::message::Message;
use zero_core::provider::{
    AnthropicLoopProvider, LoopProvider, OllamaLoopProvider, OpenAILoopProvider, StreamEvent,
    StreamingLoopProvider,
};
use zero_core::tool::{
    BashTool, EditFileTool, ReadFileTool, RegistryToolDispatcher, Tool, ToolDispatcher,
    ToolRegistry, WriteFileTool,
};

mod tui;
use tui::runtime::RuntimeEvent;

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

    /// API key (falls back to provider.api_key in ~/.zero/config.yaml)
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

    /// Disable TUI and use legacy stdio mode
    #[arg(long, default_value_t = false)]
    no_tui: bool,
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

/// Resolve the API key from CLI flag or config file.
fn resolve_api_key(cli_key: &Option<String>, config: &Config, provider: &str) -> Result<String> {
    if let Some(key) = cli_key {
        return Ok(key.clone());
    }

    let file_key = config
        .provider
        .as_ref()
        .and_then(|p| p.api_key.as_ref())
        .filter(|k| !k.trim().is_empty())
        .cloned();

    file_key.with_context(|| {
        format!(
            "API key not provided via --api-key and provider.api_key is missing in ~/.zero/config.yaml for provider '{}'",
            provider
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
fn create_provider(
    cli: &Cli,
    config: &Config,
    tool_defs: Vec<serde_json::Value>,
) -> Result<(ProviderKind, String)> {
    match cli.provider.as_str() {
        "anthropic" => {
            let api_key = resolve_api_key(&cli.api_key, config, "anthropic")?;
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
            let api_key = resolve_api_key(&cli.api_key, config, "openai")?;
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
            if let Some(ref base_url) = cli.endpoint {
                provider = provider.with_base_url(base_url);
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

    // Load unified config from ~/.zero/config.yaml (fallback to defaults)
    let zero = ZeroInit::load().unwrap_or_else(|e| {
        eprintln!("Init warning: {e}");
        ZeroInit::default()
    });
    let _ = &zero.config; // baseline config available for future use

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
    let (provider_kind, model_name) = create_provider(&cli, &zero.config, tool_defs)?;
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
            if cli.no_tui || cli.prompt.is_some() {
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
            } else {
                let history = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));
                let agent_loop = Arc::new(agent_loop);
                let config = config.clone();
                let history_clone = Arc::clone(&history);
                let loop_clone = Arc::clone(&agent_loop);
                let provider_for_ui = provider_name.clone();
                let model_for_ui = model_name.clone();

                tokio::select! {
                    result = tui::run_tui(provider_for_ui, model_for_ui, move |session_id, prompt, tx| {
                        let history = Arc::clone(&history_clone);
                        let agent_loop = Arc::clone(&loop_clone);
                        let config = config.clone();
                        tokio::spawn(async move {
                            let mut histories = history.lock().await;
                            if histories.len() <= session_id {
                                histories.resize_with(session_id + 1, Vec::new);
                            }
                            let messages = &mut histories[session_id];
                            messages.push(Message::user(prompt));
                            match agent_loop
                                .execute(messages, &config)
                                .await
                            {
                                Ok(response) => {
                                    let _ = tx.send(RuntimeEvent::TokenDelta { session_id, text: response });
                                    let _ = tx.send(RuntimeEvent::Done { session_id });
                                }
                                Err(e) => {
                                    let _ = tx.send(RuntimeEvent::Error {
                                        session_id,
                                        message: format!("Agent error: {}", e),
                                    });
                                }
                            }
                        });
                        Ok(())
                    }) => {
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
            if cli.no_tui || cli.prompt.is_some() {
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
            } else {
                let history = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));
                let agent_loop = Arc::new(agent_loop);
                let config = config.clone();
                let history_clone = Arc::clone(&history);
                let loop_clone = Arc::clone(&agent_loop);
                let provider_for_ui = provider_name.clone();
                let model_for_ui = model_name.clone();

                tokio::select! {
                    result = tui::run_tui(provider_for_ui, model_for_ui, move |session_id, prompt, tx| {
                        let history = Arc::clone(&history_clone);
                        let agent_loop = Arc::clone(&loop_clone);
                        let config = config.clone();
                        tokio::spawn(async move {
                            let mut histories = history.lock().await;
                            if histories.len() <= session_id {
                                histories.resize_with(session_id + 1, Vec::new);
                            }
                            let messages = &mut histories[session_id];
                            messages.push(Message::user(prompt));
                            let result = agent_loop
                                .execute_streaming(messages, &config, |event| match event {
                                    StreamEvent::TextDelta(text) => {
                                        let _ = tx.send(RuntimeEvent::TokenDelta { session_id, text });
                                    }
                                    StreamEvent::ToolUseStart { name, .. } => {
                                        let _ = tx.send(RuntimeEvent::ToolEvent { session_id, name });
                                    }
                                    _ => {}
                                })
                                .await;
                            match result {
                                Ok(_) => {
                                    let _ = tx.send(RuntimeEvent::Done { session_id });
                                }
                                Err(e) => {
                                    let _ = tx.send(RuntimeEvent::Error {
                                        session_id,
                                        message: format!("Agent error: {}", e),
                                    });
                                }
                            }
                        });
                        Ok(())
                    }) => {
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

#[cfg(test)]
mod tests {
    use super::{Cli, resolve_api_key};
    use clap::Parser;
    use zero_core::config::{Config, ProviderConfig};

    #[test]
    fn resolve_api_key_prefers_cli_value() {
        let config = Config {
            provider: Some(ProviderConfig {
                api_key: Some("file-key".to_string()),
                ..ProviderConfig::default()
            }),
            ..Config::default()
        };
        let key = resolve_api_key(&Some("cli-key".to_string()), &config, "openai").unwrap();
        assert_eq!(key, "cli-key");
    }

    #[test]
    fn resolve_api_key_uses_config_when_cli_missing() {
        let config = Config {
            provider: Some(ProviderConfig {
                api_key: Some("file-key".to_string()),
                ..ProviderConfig::default()
            }),
            ..Config::default()
        };
        let key = resolve_api_key(&None, &config, "anthropic").unwrap();
        assert_eq!(key, "file-key");
    }

    #[test]
    fn resolve_api_key_errors_when_missing_in_both_sources() {
        let config = Config {
            provider: Some(ProviderConfig {
                api_key: None,
                ..ProviderConfig::default()
            }),
            ..Config::default()
        };
        let err = resolve_api_key(&None, &config, "openai").unwrap_err();
        assert!(
            err.to_string()
                .contains("provider.api_key is missing in ~/.zero/config.yaml")
        );
    }

    #[test]
    fn cli_parses_no_tui_flag() {
        let cli = Cli::parse_from(["zero-cli", "--no-tui"]);
        assert!(cli.no_tui);
    }
}
