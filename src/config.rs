use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Config that can be installed locally
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Name of database within config folder
    pub database_name: String,

    /// Library name
    pub library_name: String,
}

/// Set config
pub fn set_config(init: &Config) -> anyhow::Result<()> {
    // Get config path
    let mut path = get_config_path()?;

    // Create the config path
    std::fs::create_dir_all(&path)?;

    // Add file to path
    path.push("config.toml");

    // With init data create config.toml
    let config_string = toml::to_string(&init).unwrap();

    // Save config toml
    std::fs::write(path, config_string)?;

    Ok(())
}

/// Fetch the configuration from the provided folder path
pub fn get_config() -> anyhow::Result<Config> {
    // Get config path
    let mut path = get_config_path()?;

    // Add file to path
    path.push("config.toml");

    // Read file to end
    let config = std::fs::read_to_string(path)?;

    // Deserialize
    let config: Config = toml::from_str(&config)?;
    Ok(config)
}

/// Get config path
pub fn get_config_path() -> anyhow::Result<PathBuf> {
    // Get the config file from standard location
    let mut config_path = match home::home_dir() {
        Some(path) => path,
        None => {
            return Err(anyhow!("Impossible to get your home dir!"));
        }
    };

    // Append config path to home directory
    config_path.push(".eagle-plm");

    // Return it
    Ok(config_path)
}
