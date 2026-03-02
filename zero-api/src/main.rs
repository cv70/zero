mod domain;
mod http;
mod state;

use http::routes::app;
use state::app_state::build_state;
use zero_core::config::Config;

#[tokio::main]
async fn main() {
    let zero = zero_core::ZeroInit::load().unwrap_or_else(|e| {
        eprintln!("Init warning: {e}");
        zero_core::ZeroInit::default()
    });

    let app = app(build_state());
    let bind_addr = resolve_bind_addr(&zero.config);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {bind_addr}"));
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

fn resolve_bind_addr(config: &Config) -> String {
    if let Ok(addr) = std::env::var("ZERO_API_BIND") {
        if !addr.trim().is_empty() {
            return addr;
        }
    }

    if let Some(settings) = &config.settings {
        if let Some(addr) = settings
            .get("api_bind")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        {
            return addr;
        }
    }

    let host = config
        .settings
        .as_ref()
        .and_then(|s| s.get("api_host"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0.0");
    let port = config
        .settings
        .as_ref()
        .and_then(|s| s.get("api_port"))
        .and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(str::to_string))
        })
        .unwrap_or_else(|| "3000".to_string());

    format!("{host}:{port}")
}

#[cfg(test)]
mod tests {
    use super::resolve_bind_addr;
    use std::fs;
    use zero_core::config::Config;

    #[test]
    fn resolve_bind_addr_uses_default_when_not_set() {
        // SAFETY: tests here are simple and set env only for this process.
        unsafe { std::env::remove_var("ZERO_API_BIND") };
        let cfg = Config::default();
        assert_eq!(resolve_bind_addr(&cfg), "0.0.0.0:3000");
    }

    #[test]
    fn resolve_bind_addr_uses_settings_when_present() {
        // SAFETY: tests here are simple and set env only for this process.
        unsafe { std::env::remove_var("ZERO_API_BIND") };
        let path =
            std::env::temp_dir().join(format!("zero-api-config-{}.yaml", std::process::id()));
        fs::write(&path, "settings:\n  api_bind: \"127.0.0.1:8088\"\n").unwrap();
        let cfg = Config::from_yaml_file(path.to_str().unwrap()).unwrap();
        let _ = fs::remove_file(&path);
        assert_eq!(resolve_bind_addr(&cfg), "127.0.0.1:8088");
    }
}
