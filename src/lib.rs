pub mod config;
mod constructs;
mod executor;
pub mod server;
mod sql;
pub mod storage;

pub use config::Config;
use log::*;

pub struct Instance {
    config: Config,
}

impl Instance {
    pub fn new() -> Self {
        Instance {
            config: Config::from_env(),
        }
    }

    pub async fn start(&self) {
        info!("⚙️ Launch configuration:\n{}", &self.config);
        let mut executor = executor::Executor::new();
        let executor_tx = executor.prepare_channel();

        tokio::join!(
            tokio::spawn(async move { executor.run().await }),
            server::start_server(&self.config, executor_tx),
        );
    }
}
