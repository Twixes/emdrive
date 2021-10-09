pub mod config;
mod construct;
mod query;
pub mod serve;
pub mod store;

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
        serve::start_server(&self.config).await;
    }
}
