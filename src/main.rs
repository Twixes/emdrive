use emdrive::{config, serve};
use log::*;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("emdrive=info"))
        .init();
    info!("🔢 Starting Emdrive...");
    let config = config::Config::new();
    info!("⚙️ Launch configuration:\n{}", config);
    serve::start_server(config).await;
}
