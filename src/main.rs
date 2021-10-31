use emdrive::Instance;
use human_panic::setup_panic;
use log::*;

fn main() {
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
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(instance.start());
    info!("🛑 Emdrive shut down");
}
