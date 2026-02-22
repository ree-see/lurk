use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub daemon: DaemonConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub port: Option<u16>,
    pub token: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub remote: Option<String>,
    pub token: Option<String>,
    pub local: Option<bool>,
}

pub fn load_config() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&content)?)
}

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".lurk")
        .join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.server.token.is_none());
        assert!(config.server.port.is_none());
        assert!(config.daemon.remote.is_none());
        assert!(config.daemon.token.is_none());
        assert!(config.daemon.local.is_none());
    }

    #[test]
    fn test_config_from_toml() {
        let toml_str = r#"
[server]
token = "secret"
port = 8888

[daemon]
remote = "ws://example.com:9999"
token = "secret"
local = true
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.token.as_deref(), Some("secret"));
        assert_eq!(config.server.port, Some(8888));
        assert_eq!(config.daemon.remote.as_deref(), Some("ws://example.com:9999"));
        assert_eq!(config.daemon.token.as_deref(), Some("secret"));
        assert_eq!(config.daemon.local, Some(true));
    }

    #[test]
    fn test_partial_config() {
        let toml_str = r#"
[daemon]
remote = "ws://localhost:9999"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.server.token.is_none());
        assert_eq!(config.daemon.remote.as_deref(), Some("ws://localhost:9999"));
        assert!(config.daemon.token.is_none());
    }

    #[test]
    fn test_empty_config() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.server.token.is_none());
        assert!(config.daemon.remote.is_none());
    }
}
