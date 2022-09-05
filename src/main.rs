//! src/main.rs
use hawk::log;
use hawk::models::files::File;
use hawk::models::config::Config;
use hawk::watchers;
use hawk::actions;
use hawk::cli::{Args, Action};

use std::path::Path;
use clap::Parser;
use colored::*;

fn main() -> notify::Result<()> {
    let args = Args::parse();

    let default_config_file: String = "hawk-config.yaml".to_string();
    let config_file: String = args.config.unwrap_or(default_config_file);

    let path = Path::new(&config_file);

    if !path.exists() {
        println!(
            "Canot find a valid config file ({})",
            config_file.underline().blue()
        );
        return Ok(());
    }

    let config: Config = Config::load(path).expect("Could not read config file");

    match args.action {
        None => {
            for mut workspace in config.workspaces {
                let target = config.target.clone();

                if let Err(e) = workspace.load_name_if_possible() {
                    log::error("An error has occurred:", e);
                }

                println!(
                    "Copying [{}]({}) to {}",
                    workspace.name.blue(),
                    workspace.path.underline().dimmed(),
                    target.blue()
                );
                
                actions::copy(&workspace, &target)?;
            }
        }
        Some(Action::Init(f)) => {
            if let Err(err) = actions::init(&f) {
                log::error("An error has occurred", err)
            }
        }
        Some(Action::Clean) => {
            for mut workspace in config.workspaces {
                let target = config.target.clone();

                if let Err(e) = workspace.load_name_if_possible() {
                    log::error("An error has occurred:", e);
                }

                actions::clean(workspace, &target)?;
            }
        }
        Some(Action::Copy(f)) => {
            let handle = std::thread::spawn(move || {
                if !f.watch {
                    return;
                }

                if let Err(err) = watchers::watch_config(&config_file) {
                    log::error("An error has occurred:", err)
                }
            });

            for mut workspace in config.workspaces {
                let target = config.target.clone();

                if let Err(e) = workspace.load_name_if_possible() {
                    log::error("An error has occurred:", e);
                }

                actions::copy(&workspace, &target)?;

                if f.watch {
                    std::thread::spawn(move || {
                        if let Err(err) = watchers::watch_sync(workspace, &target) {
                            log::error("Something went wrong:", err)
                        }
                    });
                }
            }

            handle.join().unwrap();
        },
    }

    Ok(())
}

