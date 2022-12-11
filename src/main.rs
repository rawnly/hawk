//! src/main.rs
use hawk_cli::actions;
use hawk_cli::cli::{Action, Args};
use hawk_cli::log;
use hawk_cli::models::config::Config;
use hawk_cli::models::environment_files::is_empty_dir;
use hawk_cli::models::files::File;
use hawk_cli::watchers;

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
        Some(Action::Init(f)) => {
            if let Err(err) = actions::init(&f) {
                log::error("An init error has occurred", err)
            }
        }
        Some(Action::Clean) => {
            let config: Config = Config::load(path).expect("Could not read config file");

            let target = Path::new(&config.target);

            if !target.exists() {
                println!("Invalid path {}", &config.target.underline().blue());
                return Ok(());
            }

            if is_empty_dir(target) {
                println!("Empty directory: {}", &config.target.underline().blue());
                return Ok(());
            }

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

            let p = Path::new(target);

            if !p.exists() {
                println!("Invalid path {}", target.underline().blue());
                return Ok(());
            }

            if is_empty_dir(p) {
                println!("Empty directory: {}", target.underline().blue());
                return Ok(());
            }

            for workspace in &config.workspaces {
                if let Some(scope) = &args.scope {
                    if scope != &workspace.name {
                        continue;
                    }
                }

                actions::list(workspace, target);
            }
        }
        _ => {
            let config: Config = Config::load(path).expect("Could not read config file");

            let handle = std::thread::spawn(move || {
                if !args.watch {
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

                println!(
                    "{} > Copying contents of {} to {}",
                    workspace.name.on_yellow().black(),
                    workspace.path.underline().dimmed(),
                    config.target.blue()
                );
                actions::copy(&workspace, &target)?;

                if args.watch {
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
