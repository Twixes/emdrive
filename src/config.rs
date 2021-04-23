
use std::{env, fmt, str::FromStr};

pub fn get_env_or(key: String, default: String) -> String {
    std::env::var(key).unwrap_or(default)
}

pub fn get_env_cast_or<T: FromStr + fmt::Display>(key: String, default: T) -> T {
    let value_raw = &env::var(&key);
    if let Ok(value_raw) = value_raw {
        match T::from_str(value_raw) {
            Ok(value) => value,
            Err(_) => panic!("{} is not a valid {} value!", value_raw, key)
        }
    } else {
        default
    }
}
