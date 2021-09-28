use std::{env, fmt, str};

/// DBMS configuration.
#[derive(Debug)]
pub struct Config {
    /// Path to database state, i.e. saved data. Conventionally `emdrive/` in `/var/lib/`.
    pub data_directory: String,
    /// TCP interface listener host, `127.0.0.1` by default.
    pub tcp_listen_host: String,
    /// TCP interface listener port. `8824` by default.
    pub tcp_listen_port: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            data_directory: "/var/lib/emdrive/data".to_string(),
            tcp_listen_host: "127.0.0.1".to_string(),
            tcp_listen_port: 8824,
        }
    }
}

impl Config {
    pub fn new() -> Config {
        let default = Config::default();
        Config {
            data_directory: get_env_or("data_directory", default.data_directory),
            tcp_listen_host: get_env_or("tcp_listen_host", default.tcp_listen_host),
            tcp_listen_port: get_env_cast_or("tcp_listen_port", default.tcp_listen_port),
        }
    }
}

fn get_env(key: &str) -> Result<String, env::VarError> {
    env::var(format!("EMDRIVE_{}", &key.to_uppercase()))
}

fn get_env_or(key: &str, default: String) -> String {
    get_env(key).unwrap_or(default)
}

fn get_env_cast_or<T: str::FromStr + fmt::Display>(key: &str, default: T) -> T {
    let value_raw = get_env(key);
    if let Ok(value_raw) = value_raw {
        match T::from_str(&value_raw) {
            Ok(value) => value,
            Err(_) => panic!("{} is not a valid {} value!", value_raw, key),
        }
    } else {
        default
    }
}
