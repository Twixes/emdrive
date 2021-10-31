use std::{env, fmt, str};

/// DBMS configuration.
#[derive(Debug, Clone)]
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

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}={:?}\n{}={:?}\n{}={:?}",
            envify_config_key("data_directory"),
            self.data_directory,
            envify_config_key("tcp_listen_host"),
            self.tcp_listen_host,
            envify_config_key("tcp_listen_port"),
            self.tcp_listen_port
        )
    }
}

impl Config {
    pub fn from_env() -> Config {
        let default = Config::default();
        Config {
            data_directory: get_env_or("data_directory", default.data_directory),
            tcp_listen_host: get_env_or("tcp_listen_host", default.tcp_listen_host),
            tcp_listen_port: get_env_cast_or("tcp_listen_port", default.tcp_listen_port),
        }
    }
}

// Format internal config key to environment variable name.
fn envify_config_key(key: &str) -> String {
    format!("EMDRIVE_{}", &key.to_uppercase())
}

fn get_env(key: &str) -> Result<String, env::VarError> {
    env::var(envify_config_key(key))
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
