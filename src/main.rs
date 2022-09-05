//! src/main.rs
use hawk::actions;
use hawk::cli::{Action, Args};
use hawk::log;
use hawk::models::config::Config;
use hawk::models::files::File;
use hawk::watchers;

use clap::Parser;
use colored::*;
use std::path::Path;

fn get_config_path(args: Args) -> String {
    let default_config_file: String = "hawk-config.yaml".to_string();
    args.config.unwrap_or(default_config_file)
}

fn main() -> notify::Result<()> {
    let args = Args::parse();
    let config_file = get_config_path(args.clone());
    let path = Path::new(&config_file);

    if !matches!(args.action, Some(Action::Init(_))) && !path.exists() {
        println!(
            "Canot find a valid config file ({})",
            config_file.underline().blue()
        );

        return Ok(());
    }

    match args.action {
        None => {
            let config: Config = Config::load(path).expect("Could not read config file");

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
            let config: Config = Config::load(path).expect("Could not read config file");

            for mut workspace in config.workspaces {
                let target = config.target.clone();

                if let Err(e) = workspace.load_name_if_possible() {
                    log::error("An error has occurred:", e);
                }

                actions::clean(workspace, &target)?;
            }
        }
        Some(Action::Copy(f)) => {
            let config: Config = Config::load(path).expect("Could not read config file");

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
        }
    }

    Ok(())
}
