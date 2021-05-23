use metrobaza::{config, tcp};

fn main() {
    let config = config::Config::new();
    tcp::start_server(config);
}
