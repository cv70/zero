#[tokio::main]
async fn main() {
    let shadow_mode = std::env::var("ZERO_SHADOW_MODE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if shadow_mode {
        println!("Zero API Server (shadow mode enabled)");
    } else {
        println!("Zero API Server");
    }
}
