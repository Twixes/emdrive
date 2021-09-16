use crate::data;
use crate::{config, timeprintln};
use futures::prelude::*;
use std::{net, str::FromStr};

pub fn say_hello(state: gotham::state::State) -> (gotham::state::State, String) {
    timeprintln!("Hello");
    (state, "Hello!".to_string())
}

/// Start Metrobaza server loop.
pub async fn start_server(config: config::Config) {
    // TCP server starting
    let tcp_listen_address = net::SocketAddr::new(
        net::IpAddr::from_str(&config.tcp_listen_host).unwrap(),
        config.tcp_listen_port,
    );
    let server = gotham::init_server(tcp_listen_address, || Ok(say_hello));
    // Future to wait for Ctrl+C.
    let signal = async {
        tokio::signal::ctrl_c().map_err(|_| ()).await?;
        println!("Ctrl+C pressed");
        Ok::<(), ()>(())
    };

    future::select(server.boxed(), signal.boxed()).await;
    println!("Shutting down gracefully");
}
