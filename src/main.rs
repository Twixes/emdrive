use emdrive::{config, server, timeprintln};

#[tokio::main]
async fn main() {
    timeprintln!("🔢 Starting Emdrive...");
    let config = config::Config::new();
    timeprintln!("⚙️ Launch configuration:\n{}", config);
    server::start_server(config).await;
}
