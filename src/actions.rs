use colored::*;
use std::path::Path;
use std::fs;

use crate::cli::InitFlags;
use crate::models::config::Config;
use crate::models::environment_files::list_files;
use crate::models::files;
use crate::models::files::*;
use crate::models::workflow::Workflow;
use crate::models::workspace::Workspace;
use crate::utils;

pub fn list(workspace: &Workspace, target: &str) {
    list_files(Path::new(target))
        .iter()
        .filter(|f| utils::is_workflow_file(*f) && f.file_name().unwrap().to_str().unwrap_or("").starts_with(&workspace.name))
        .map(|f| (Workflow::load(f).unwrap(), f.file_name().unwrap().to_str().unwrap_or("")))
        .for_each(|(w, p)| {
            println!("{}: {}", w.name.bold().cyan(), p);
        })
}

pub fn init(flags: &InitFlags) -> files::Result<Config> {
    let mut config_path = Path::new("hawk-config.yaml").to_path_buf();

    if flags.json {
        config_path = config_path.with_extension("json");
    }

    if flags.read_env {
        let wk_target = flags
            .clone()
            .workflows
            .unwrap_or_else(|| ".github/workflows".into());
        let config = Config::init(".github/workflows", &wk_target)?;
        config.write(config_path.as_path())?;

        return Ok(config);
    }

    let config = Config::new(".github/workflows");
    config.write(config_path.as_path())?;

    Ok(config)
}

pub fn clean(workspace: Workspace, target: &str) -> std::io::Result<()> {
    if let Ok(files) = fs::read_dir(workspace.path) {
        for file in files {
            match file {
                Ok(f) => {
                    if let Some(filename) = f.path().file_name().unwrap().to_str() {
                        if utils::is_workflow_file(Path::new(filename)) {
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
                        if utils::is_workflow_file(Path::new(filename)) {
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
