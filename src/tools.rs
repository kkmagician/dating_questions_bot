use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::time::SystemTime;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn read_key_env(key: &str) -> Option<String> {
    env::var_os(key).and_then(|v| v.into_string().ok())
}

#[macro_export]
macro_rules! ternary {
    ($c:expr, $v:expr, $v1:expr) => {
        if $c {
            $v
        } else {
            $v1
        }
    };
}

pub fn get_parse_string_value<T: FromStr>(
    hm: &HashMap<String, String>,
    key: &str,
    default: T,
) -> T {
    hm.get(key).and_then(|x| x.parse().ok()).unwrap_or(default)
}

pub fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn random_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
