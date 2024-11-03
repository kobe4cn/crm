use std::fs::File;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub pk: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    // pub host: String,
    pub port: u16,
    pub db_url: String,
    pub duck_db: String,
}

impl AppConfig {
    pub fn try_load() -> Result<Self> {
        // read from ./app.yml, or /etc/config/app.yml, or from env CHAT_CONFIG
        let ret = match (
            File::open("../user-state/user_stat.yaml"),
            File::open("user_stat.yaml"),
            File::open("/etc/config/user_stat.yaml"),
            std::env::var("NOTIFY_CONFIG"),
        ) {
            (Ok(reader), _, _, _) => serde_yaml::from_reader(reader),
            (_, Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (_, _, Ok(reader), _) => serde_yaml::from_reader(reader),
            (_, _, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("No config file found"),
        };

        Ok(ret?)
    }
}
