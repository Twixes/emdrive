
use std::{env, fmt, str};

#[derive(Debug)]
pub struct Config {
    pub state_location: String,
    pub tcp_listen_host: String,
    pub tcp_listen_port: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            state_location: "/var/lib/metrobaza/".to_string(),
            tcp_listen_host: "127.0.0.1".to_string(),
            tcp_listen_port: 8824
        }
    }
}

impl Config {
    pub fn new() -> Config {
        let default = Config::default();
        Config {
            state_location: get_env_or("STATE_LOCATION", default.state_location),
            tcp_listen_host: get_env_or("TCP_LISTEN_HOST", default.tcp_listen_host),
            tcp_listen_port: get_env_cast_or("TCP_LISTEN_PORT", default.tcp_listen_port)
        }
    }
}

fn get_env(key: &str) -> Result<String, env::VarError> {
    env::var(format!("METRO_{}", &key.to_uppercase()))
}

fn get_env_or(key: &str, default: String) -> String {
    get_env(key).unwrap_or(default)
}

fn get_env_cast_or<T: str::FromStr + fmt::Display>(key: &str, default: T) -> T {
    let value_raw = get_env(key);
    if let Ok(value_raw) = value_raw {
        match T::from_str(&value_raw) {
            Ok(value) => value,
            Err(_) => panic!("{} is not a valid {} value!", value_raw, key)
        }
    } else {
        default
    }
}
