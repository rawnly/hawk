use colored::*;
use std::fs;
use std::path::Path;

use crate::cli::InitFlags;
use crate::models::config::Config;
use crate::models::environment_files::{is_empty_dir, list_files};
use crate::models::files;
use crate::models::files::*;
use crate::models::workflow::Workflow;
use crate::models::workspace::Workspace;
use crate::utils;

pub fn list(workspace: &Workspace, target: &str) {
    let t = Path::new(target);

    list_files(t)
        .iter()
        .filter(|f| {
            utils::is_workflow_file(f)
                && f.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap_or("")
                    .starts_with(&workspace.name)
        })
        .map(|f| {
            (
                Workflow::load(f).expect("invalid workflow file"),
                f.file_name().unwrap().to_str().unwrap_or(""),
            )
        })
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

    println!("Project setup completed!");

    Ok(config)
}

pub fn clean(workspace: Workspace, target: &str) -> std::io::Result<()> {
    if let Ok(files) = fs::read_dir(workspace.path) {
        for file in files {
            match file {
                Ok(f) => {
                    if utils::is_workflow_file(&f.path()) {
                        utils::remove_file(&f.path(), target, &workspace.name)?;

                        println!(
                            "Removing {}",
                            utils::target_filename(&f.path(), target, &workspace.name)
                                .underline()
                                .blue()
                        );
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
                    let is_workflow = utils::is_workflow_file(&path.path());

                    if is_workflow {
                        utils::copy_file(&path.path(), target, &workspace.name)?
                    } else {
                        println!("Skipping: {:?} {}", path.path().display(), is_workflow);
                        skipped += 1;
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
