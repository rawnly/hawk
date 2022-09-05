use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

use crate::models::environment_files::{list_dirs, search_file, PackageJson, PnpmWorkspace};
use crate::models::files;
use crate::models::files::File;
use crate::models::workspace;

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    YamlError(serde_yaml::Error),
    FileNotFound,
    Any,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub workspaces: Vec<workspace::Workspace>,
    pub target: String,
}

impl Config {
    #[deprecated(since = "1.0.3")]
    pub fn load_old(filepath: &str) -> Result<Config> {
        if !Path::new(filepath).exists() {
            return Err(ConfigError::FileNotFound);
        }

        let f = fs::File::open(filepath).expect("Unable to open file");

        match serde_yaml::from_reader(f) {
            Ok(data) => Ok(data),
            Err(e) => Err(ConfigError::YamlError(e)),
        }
    }

    #[deprecated(since = "1.0.3")]
    pub fn validate_workspaces(&self) -> workspace::Result<()> {
        for workspace in &self.workspaces {
            workspace.validate_name()?;
            workspace.validate_path()?;
        }

        Ok(())
    }

    pub fn new(target: &str) -> Config {
        Config {
            target: target.into(),
            workspaces: Vec::new(),
        }
    }

    /// Initialize config reading workspaces from package.json or `pnpm-workspace` if available.
    /// `pnpm-workspace` has priority over package.json workspaces key.
    pub fn init(target: &str, workflows_dir: &str) -> files::Result<Config> {
        let mut config = Config::new(target);
        let package_json_path = search_file(".", "package.json");
        let pnpm_workspace_path = search_file(".", "pnpm-workspace.yaml");

        dbg!(&package_json_path, &pnpm_workspace_path);

        if pnpm_workspace_path.is_some() {
            let mut workspaces: Vec<workspace::Workspace> = Vec::new();
            let pnpm_workspace: PnpmWorkspace = PnpmWorkspace::load(&pnpm_workspace_path.unwrap())?;

            for el in pnpm_workspace.packages {
                add_workspaces(
                    &mut workspaces,
                    Path::new(&format!("./{}", el.replace("/*", ""))),
                    workflows_dir,
                )?;
            }

            config.workspaces = workspaces;

            return Ok(config);
        }

        if let Some(path) = package_json_path {
            let package_json = PackageJson::load(&path)?;
            let mut workspaces: Vec<workspace::Workspace> = Vec::new();

            if !package_json.clone().has_workspaces() {
                return Ok(config);
            }

            for el in package_json.workspaces.unwrap() {
                add_workspaces(
                    &mut workspaces,
                    Path::new(&format!("./{}", el.replace("/*", ""))),
                    workflows_dir,
                )?;
            }

            config.workspaces = workspaces;
        }

        dbg!(config.clone());
        Ok(config)
    }
}

fn add_workspaces(
    workspaces: &mut Vec<workspace::Workspace>,
    directory: &Path,
    workflows_dir: &str,
) -> files::Result<()> {
    for dir in list_dirs(directory) {
        dbg!(&dir, &directory);
        if search_file(dir.to_str().unwrap(), "package.json").is_some() {
            dbg!(dir.join("package.json"));
            let pkg = PackageJson::load(dir.join("package.json").as_path())
                .expect("could not find package json");

            let wk = workspace::Workspace {
                name: pkg.name,
                package_json: Some(dir.to_str().unwrap().into()),
                path: dir.join(workflows_dir).to_str().unwrap_or_default().into(),
            };

            workspaces.push(wk);
        } else {
            workspaces.push(workspace::Workspace {
                name: dir.file_stem().unwrap().to_str().unwrap().into(),
                package_json: None,
                path: dir.join(workflows_dir).to_str().unwrap_or_default().into(),
            });
        }
    }

    Ok(())
}

impl File<Config> for Config {}
