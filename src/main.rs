use std::str::FromStr;

use emdrive::Instance;
use human_panic::setup_panic;
use tracing::*;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn main() {
    setup_panic!(Metadata {
        name: "Emdrive".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "".into(), // Empty to disable
        homepage: env!("CARGO_PKG_REPOSITORY").into(),
    });
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_str(&"emdrive=debug").unwrap())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("🔢 Starting Emdrive...");
    let instance = Instance::preload();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(instance.run()).unwrap();
    info!("🛑 Emdrive shut down");
}
