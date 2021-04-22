use std::{net::{IpAddr, SocketAddr, TcpListener}, str::FromStr};

fn get_env_or(key: String, default: String) -> String {
    match std::env::var_os(key) {
        Some(value) => value.into_string().unwrap(),
        None => default,
    }
}

fn get_env_cast_or<T: std::str::FromStr>(key: String, default: T) -> T {
    match std::env::var_os(key) {
        Some(value) => T::from_str(value.to_str().unwrap()).ok().unwrap(),
        None => default,
    }
}

const TCP_LISTEN_HOST_DEFAULT: &str = "127.0.0.1";
const TCP_LISTEN_PORT_DEFAULT: u16 = 8824;

fn main() {
    let tcp_listen_host = get_env_or(
        "FAKSO_TCP_LISTEN_HOST".to_string(),
        TCP_LISTEN_HOST_DEFAULT.to_string(),
    );
    let tcp_listen_port = get_env_cast_or(
        "FAKSO_TCP_LISTEN_PORT".to_string(),
        TCP_LISTEN_PORT_DEFAULT,
    );

    let tcp_listen_address = SocketAddr::new(IpAddr::from_str(&tcp_listen_host).unwrap(), tcp_listen_port);

    let listener = TcpListener::bind(tcp_listen_address).unwrap();

    for stream in listener.incoming() {
        let _stream = stream.unwrap();

        println!("Connection established!");
    }
}
