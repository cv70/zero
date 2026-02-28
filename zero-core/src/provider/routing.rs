use crate::provider::ProviderKind;

pub fn route_model(model: &str) -> ProviderKind {
    let m = model.to_ascii_lowercase();
    if m.starts_with("gpt-") || m.contains("openai") {
        ProviderKind::OpenAI
    } else if m.contains("claude") {
        ProviderKind::Anthropic
    } else if m.starts_with("ollama-") || m.contains("llama") {
        ProviderKind::Ollama
    } else {
        ProviderKind::Unknown
    }
}
