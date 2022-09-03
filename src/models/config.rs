use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::Path;

use crate::models::workspace;

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    YamlError(serde_yaml::Error),
    FileNotFound,
    Any
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::FileNotFound => write!(f, "Could not find config file"),
            ConfigError::YamlError(err) => {
                write!(f, "{}", err)
            }
            ConfigError::Any => {
                write!(f, "Something went wrong")
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub workspaces: Vec<workspace::Workspace>,
    pub target: String,
}

impl Config {
    pub fn load(filepath: &str) -> Result<Config> {
        if !Path::new(filepath).exists() {
            return Err(ConfigError::FileNotFound);
        }

        let f = fs::File::open(filepath).expect("Unable to open file");
        
        match serde_yaml::from_reader(f) {
            Ok(data) => Ok(data),
            Err(e) => Err(ConfigError::YamlError(e)),
        }
    }

    pub fn validate_workspaces(&self) -> workspace::Result<()> {
        for workspace in &self.workspaces {
            workspace.validate_name()?;
            workspace.validate_path()?;
        }

        Ok(())
    }
}
