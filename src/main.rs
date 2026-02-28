use std::error::Error;

#[allow(dead_code)]
mod security;
#[allow(dead_code)]
mod network;
#[allow(dead_code)]
mod storage;
#[allow(dead_code)]
mod auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Simple bootstrap
    env_logger::init();

    // Demo: Network input validation and processing
    let net_input = r#"{"address":"127.0.0.1","port":8080,"payload":"ping"}"#;
    match network::process(net_input) {
        Ok(out) => println!("NetworkOutput: {:?}", out),
        Err(e) => eprintln!("Network error: {}", e),
    }

    // Demo: Storage input validation
    let store_input = r#"{"key":"user:1:name","value":"Alice"}"#;
    match storage::put(store_input) {
        Ok(out) => println!("StorageOutput: {:?}", out),
        Err(e) => eprintln!("Storage error: {}", e),
    }

    // Demo: Auth input validation
    let login_input = r#"{"username":"alice","password":"s3cretp@ss"}"#;
    match auth::login(login_input) {
        Ok(token) => println!("Auth token: {}", token),
        Err(e) => eprintln!("Auth error: {}", e),
    }

    // Vulnerability scanning (best-effort)
    if let Err(e) = security::scan::run_vulnerability_scan() {
        eprintln!("Security scan failed: {}", e);
    }

    // Security monitoring startup (background task in real app)
    // security::monitor::start_monitoring().await;

    Ok(())
}
