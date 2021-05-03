use std::{net, str::FromStr};

mod config;

fn start_listener(tcp_listen_address: net::SocketAddr) -> net::TcpListener {
    let listener = net::TcpListener::bind(tcp_listen_address).unwrap();
    let local_addr = listener.local_addr().unwrap();
    println!("TCP server listening on {}...", local_addr);
    listener
}

fn main() {
    let config = config::Config::new();

    let tcp_listen_address = net::SocketAddr::new(net::IpAddr::from_str(&config.tcp_listen_host).unwrap(), config.tcp_listen_port);

    let listener = start_listener(tcp_listen_address);

    for stream in listener.incoming() {
        let _stream = stream.unwrap();

        println!("Connection established!");
    }
}
