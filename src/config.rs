use colored::Colorize;
use serde::Deserialize;
use std::fmt;
use std::fs;

use crate::log;

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub path: String,
    pub package_json: Option<String>,
}

pub type Result<T> = std::result::Result<T, WorkspaceError>;

#[derive(Debug, Clone)]
pub enum WorkspaceError {
    InvalidName(String),
    InvalidPath(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageJson {
    pub name: String,
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WorkspaceError::InvalidPath(path) => {
                write!(f, "Workspace path ({}) does not exist", path)
            }
            WorkspaceError::InvalidName(name) => {
                write!(f, "Workspace name ({}) cannot contain spaces", name)
            }
        }
    }
}

impl Workspace {
    pub fn validate_name(&self) -> Result<()> {
        if self.name.contains(' ') {
            return Err(WorkspaceError::InvalidName(self.name.clone()));
        }

        Ok(())
    }

    pub fn validate_path(&self) -> Result<()> {
        if !std::path::Path::new(&self.path).exists() {
            return Err(WorkspaceError::InvalidPath(self.path.clone()));
        }

        Ok(())
    }

    pub fn load_name_if_possible(&mut self) -> serde_json::Result<()> {
        if let Some(path) = &self.package_json {
            let mut p: String = path.into();

            if std::path::Path::new(path).is_dir() {
                p = format!("{}/package.json", path);
            }

            if !std::path::Path::new(&p).exists() {
                log::warn(&format!(
                    "Could not load package.json name for workspace: {}. Cannot find file.",
                    self.name.underline()
                ));
                return Ok(());
            }

            let f = fs::File::open(p).expect("Unable to open package.json");
            let pkg: PackageJson = serde_json::from_reader(f)?;

            self.name = pkg.name;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub workspaces: Vec<Workspace>,
    pub target: String,
}

impl Config {
    pub fn load(filepath: &str) -> serde_yaml::Result<Config> {
        let f = fs::File::open(filepath).expect("Unable to open file");
        serde_yaml::from_reader(f)
    }

    pub fn validate_workspaces(&self) -> Result<()> {
        for workspace in &self.workspaces {
            workspace.validate_name()?;
            workspace.validate_path()?;
        }

        Ok(())
    }
}
