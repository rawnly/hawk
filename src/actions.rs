use colored::*;
use std::{fs, path::Path};

use crate::cli::InitFlags;
use crate::models::config::Config;
use crate::models::files;
use crate::models::files::*;
use crate::models::workspace::Workspace;
use crate::utils;

pub fn init(flags: &InitFlags) -> files::Result<Config> {
    dbg!("FLAGS", flags);

    let mut config_path = Path::new("hawk-config.yaml").to_path_buf();
    let target = flags
        .clone()
        .target
        .unwrap_or_else(|| ".github/workflows".to_string());

    if flags.json {
        config_path = config_path.with_extension("json");
    }

    if flags.read_env {
        let config = Config::init(&target)?;
        config.write(config_path.as_path())?;

        return Ok(config);
    }

    let config = Config::new(&target);
    config.write(config_path.as_path())?;

    Ok(config)
}

pub fn clean(workspace: Workspace, target: &str) -> std::io::Result<()> {
    if let Ok(files) = fs::read_dir(workspace.path) {
        for file in files {
            match file {
                Ok(f) => {
                    if let Some(filename) = f.path().file_name().unwrap().to_str() {
                        if f.path().exists() && f.path().is_file() && utils::is_yaml(filename) {
                            utils::remove_file(&f.path(), target, &workspace.name)?;
                            println!(
                                "Removing {}",
                                utils::target_filename(&f.path(), target, &workspace.name)
                                    .underline()
                                    .blue()
                            );
                        }
                    }
                }
                Err(err) => println!("Failed to delete file: {}", err),
            }
        }
    }

    Ok(())
}

pub fn copy(workspace: &Workspace, target: &str) -> notify::Result<()> {
    let mut skipped = 0;

    if let Ok(content) = fs::read_dir(&workspace.path) {
        for f in content {
            match f {
                Ok(path) => {
                    if let Some(filename) = path.path().file_name().unwrap().to_str() {
                        if path.path().is_file() && utils::is_yaml(filename) {
                            utils::copy_file(&path.path(), target, &workspace.name)?
                        } else {
                            println!(
                                "Skipping: {:?} {}",
                                path.path().display(),
                                path.path()
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .ends_with("yml")
                            );
                            skipped += 1;
                        }
                    }
                }
                Err(err) => println!("failed to copy: {}", err),
            }
        }
    }

    if skipped > 0 {
        println!("Skipped {} files.", skipped);
    }

    Ok(())
}
