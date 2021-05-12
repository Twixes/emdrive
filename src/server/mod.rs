use crate::config;
use std::{net, str::FromStr};

fn create_listener(tcp_listen_address: net::SocketAddr) -> net::TcpListener {
    let listener = net::TcpListener::bind(tcp_listen_address).unwrap();
    let local_addr = listener.local_addr().unwrap();
    println!("TCP server listening on {}...", local_addr);
    listener
}

pub fn start_server(config: config::Config) {
    let tcp_listen_address = net::SocketAddr::new(
        net::IpAddr::from_str(&config.tcp_listen_host).unwrap(),
        config.tcp_listen_port,
    );

    let listener = create_listener(tcp_listen_address);

    for stream in listener.incoming() {
        let _stream = stream.unwrap();

        println!("Connection established!");
    }
}
