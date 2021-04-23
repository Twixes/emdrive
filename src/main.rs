use std::{net, str::FromStr};

mod config;

const TCP_LISTEN_HOST_DEFAULT: &str = "127.0.0.1";
const TCP_LISTEN_PORT_DEFAULT: u16 = 8824;

fn start_listener(tcp_listen_address: net::SocketAddr) -> net::TcpListener {
    let listener = net::TcpListener::bind(tcp_listen_address).unwrap();
    let local_addr = listener.local_addr().unwrap();
    println!("TCP server listening on {}...", local_addr);
    listener
}

fn main() {
    let tcp_listen_host = config::get_env_or(
        "FAKSO_TCP_LISTEN_HOST".to_string(),
        TCP_LISTEN_HOST_DEFAULT.to_string(),
    );
    let tcp_listen_port = config::get_env_cast_or(
        "FAKSO_TCP_LISTEN_PORT".to_string(),
        TCP_LISTEN_PORT_DEFAULT,
    );

    let tcp_listen_address = net::SocketAddr::new(net::IpAddr::from_str(&tcp_listen_host).unwrap(), tcp_listen_port);

    let listener = start_listener(tcp_listen_address);

    for stream in listener.incoming() {
        let _stream = stream.unwrap();

        println!("Connection established!");
    }
}
