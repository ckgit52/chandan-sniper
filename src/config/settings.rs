use std::env;

pub struct Config {
    pub rpc_url: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            rpc_url: env::var("RPC_URL").unwrap_or_default(),
        }
    }
}
