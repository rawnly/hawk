use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::files::File;

#[derive(Debug, Deserialize, Clone)]
pub struct PackageJson {
    pub name: String,
    pub workspaces: Option<Vec<String>>,
}

impl File<PackageJson> for PackageJson {}
impl PackageJson {
    pub fn has_workspaces(self) -> bool {
        self.workspaces.unwrap().is_empty()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PnpmWorkspace {
    pub packages: Vec<String>,
}
impl File<PnpmWorkspace> for PnpmWorkspace {}

pub fn search_file(search_path: &str, filename: &str) -> Option<PathBuf> {
    match fs::read_dir(search_path) {
        Err(_) => None,
        Ok(content) => {
            let mut fpath: Option<PathBuf> = None;

            for f in content {
                let path = match f {
                    Err(_) => None,
                    Ok(entry) => Some(entry.path()),
                };

                if let Some(p) = path {
                    if p.is_file() && p.file_name().unwrap().to_str().unwrap() == filename {
                        fpath = Some(p);
                        break;
                    }
                }
            }

            fpath
        }
    }
}

pub fn list_dirs(path: &Path) -> Vec<PathBuf> {
    match fs::read_dir(path) {
        Err(_) => Vec::new(),
        Ok(content) => content
            .filter_map(move |p| match p {
                Ok(entry) => Some(entry.path()),
                _ => None,
            })
            .filter(|p| p.is_dir())
            .collect(),
    }
}

pub fn list_files(path: &Path) -> Vec<PathBuf> {
    match fs::read_dir(path) {
        Err(_) => Vec::new(),
        Ok(content) => content
            .filter_map(move |p| match p {
                Ok(entry) => Some(entry.path()),
                _ => None,
            })
            .filter(|p| p.is_file())
            .collect(),
    }
}
