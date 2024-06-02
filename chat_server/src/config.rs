use std::{env, fs::File};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

impl AppConfig {
    pub fn try_load() -> Result<Self> {
        // read from /etc/config/app.yml, or ./app.yml or from env CHAT_CONFIG
        let ret = match (
            env::var("CHAT_CONFIG"),
            File::open("app.yml"),
            File::open("/etc/config/app.yml"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(File::open(file)?),
            (_, Ok(file), _) => serde_yaml::from_reader(file),
            (_, _, Ok(file)) => serde_yaml::from_reader(file),
            // _ => return Err(anyhow::anyhow!("Config file not found")),
            _ => bail!("Config file not found"),
        };
        Ok(ret?)
    }
}
