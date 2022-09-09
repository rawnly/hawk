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

fn main() -> notify::Result<()> {
    let args = Args::parse();
    let config_file = match args.config {
        Some(c) => c,
        None => "hawk-config.yaml".into(),
    };

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

            for workspace in config.workspaces {
                if let Some(scope) = &args.scope {
                    if scope != &workspace.name {
                        continue;
                    }
                }

                println!(
                    "Copying [{}]({}) to {}",
                    workspace.name.blue(),
                    workspace.path.underline().dimmed(),
                    config.target.blue()
                );

                actions::copy(&workspace, &config.target)?;
            }
        }
        Some(Action::Init(f)) => {
            if let Err(err) = actions::init(&f) {
                log::error("An init error has occurred", err)
            }
        }
        Some(Action::Clean) => {
            let config: Config = Config::load(path).expect("Could not read config file");

            for workspace in config.workspaces {
                if let Some(scope) = &args.scope {
                    if scope != &workspace.name {
                        continue;
                    }
                }

                actions::clean(workspace, &config.target)?;
            }
        }
        Some(Action::List) => {
            let config: Config = Config::load(path).expect("Could not read config file");
            let target = &config.target;

            for workspace in &config.workspaces {
                if let Some(scope) = &args.scope {
                    if scope != &workspace.name {
                        continue;
                    }
                }

                actions::list(workspace, target);
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

            for workspace in config.workspaces {
                if let Some(scope) = &args.scope {
                    if scope != &workspace.name {
                        continue;
                    }
                }

                let target = config.target.clone();

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
