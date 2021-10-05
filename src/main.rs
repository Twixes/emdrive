use emdrive::{config, server, timeprintln};

#[tokio::main]
async fn main() {
    timeprintln!("🔢 Starting Emdrive...");
    let config = config::Config::new();
    timeprintln!("⚙️ Configuration:\n{}", config);
    server::start_server(config).await;
}
