use faksobaza::{config, server};

fn main() {
    let config = config::Config::new();
    server::start_server(config);
}
