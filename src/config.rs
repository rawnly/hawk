use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub path: String
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
}

