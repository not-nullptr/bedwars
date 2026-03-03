use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub server: ServerConfig,
    pub registry: RegistryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub bind_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub max_players: u32,
    pub motd: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryConfig {
    pub path: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, figment::Error> {
        Figment::new().merge(Toml::file(path)).extract()
    }
}
