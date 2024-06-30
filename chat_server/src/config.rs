use std::{env, fs::File, path::PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
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
