use emdrive::{config, tcp};

#[tokio::main]
async fn main() {
    let config = config::Config::new();
    tcp::start_server(config).await;
}
