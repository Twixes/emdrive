use emdrive::Instance;
use human_panic::setup_panic;
use log::*;

#[tokio::main]
async fn main() {
    setup_panic!(Metadata {
        name: "Emdrive".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "".into(),
        homepage: env!("CARGO_PKG_REPOSITORY").into(),
    });
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("emdrive=debug"))
        .init();
    info!("🔢 Starting Emdrive...");
    let instance = Instance::new();
    instance.start().await;
}
