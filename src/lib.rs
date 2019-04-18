#![cfg(target_os = "linux")]

mod key_logger;
mod server;

pub use self::key_logger::{log_keys, Key, State};
pub use self::server::run_server;
