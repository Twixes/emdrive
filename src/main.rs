use emdrive::{config, tcp, timeprintln};

#[tokio::main]
async fn main() {
    timeprintln!("🔢 Starting Emdrive…");
    let config = config::Config::new();
    timeprintln!("⚙️ Configuration:\n{}", config);
    tcp::start_server(config).await;
}
