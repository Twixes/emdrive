pub mod config;
mod constructs;
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
        server::start_server(&self.config).await;
    }
}
