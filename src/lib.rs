pub mod config;
mod constructs;
mod executor;
pub mod server;
mod sql;
pub mod storage;

pub use config::Config;
use tracing::*;

pub struct Instance {
    config: Config,
}

impl Instance {
    pub fn preload() -> Self {
        Instance {
            config: Config::from_env(),
        }
    }

    pub async fn run(&self) {
        info!("⚙️ Launch configuration:\n{}", &self.config);
        let mut executor = executor::Executor::new();
        let executor_tx = executor.prepare_channel();
        let (executor_join_result, _) = tokio::join!(
            tokio::spawn(async move {
                executor.start().await;
            }),
            server::start_server(&self.config, executor_tx),
        );
        executor_join_result.expect("Failed to join executor");
    }
}
