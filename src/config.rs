use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[derive(Debug, Serialize, Deserialize)]
pub enum AttritionType {
    Each,
    Percentage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttritionEntry {
    pub value: u32,
    pub attype: AttritionType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttritionConfig {
    pub entries: Vec<AttritionEntry>,
}

/// Config that can be installed locally
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Name of database within config folder
    pub database_name: String,

    /// Library name
    pub library_name: String,

    /// Attrition config
    pub attrition_config: AttritionConfig,

    /// Ignore parts that contain one of these
    pub part_number_ignore_list: Vec<String>,
}

/// Set config
pub fn save_config(config: &Config, config_path: &Path) -> anyhow::Result<()> {
    // With init data create config.toml
    let config_string = toml::to_string(config).unwrap();

    // Save config toml
    std::fs::write(config_path, config_string)?;

    Ok(())
}

/// Fetch the configuration from the provided folder path
pub fn load_config(config_path: &Path) -> anyhow::Result<Config> {
    // Read file to end
    let config = std::fs::read_to_string(&config_path)?;

    // Deserialize
    Ok(toml::from_str(&config)?)
}

/// Calculate config path depending on input
pub fn get_config_path(config_path: &Option<String>) -> anyhow::Result<PathBuf> {
    match config_path {
        Some(c) => Ok(PathBuf::from(c)),
        None => {
            // Get config path
            let mut path = get_default_config_path()?;

            // Create the config path
            std::fs::create_dir_all(&path)?;

            // Add file to path
            path.push("config.toml");

            // Return this guy
            Ok(path)
        }
    }
}

/// Get default config path. ($HOME/.eagle-plm)
pub fn get_default_config_path() -> anyhow::Result<PathBuf> {
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
