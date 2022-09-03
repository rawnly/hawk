//! src/main.rs
use hawk::log;
use hawk::models::config::Config;
use hawk::utils;
use hawk::watchers;
use std::fs;

use clap::Parser;
use colored::*;

#[derive(clap::Subcommand, Debug)]
enum Action {
    Clean,
    Copy,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, value_parser, default_value_t = false)]
    watch: bool,

    #[clap(short, long, value_parser)]
    config: Option<String>,

    #[clap(subcommand)]
    action: Option<Action>,
}

fn is_yaml(path: &str) -> bool {
    path.ends_with("yml") || path.ends_with("yaml")
}

fn main() -> notify::Result<()> {
    let args = Args::parse();

    let default_config_file: String = "hawk-config.yaml".to_string();
    let config_file: String = args.config.unwrap_or(default_config_file);

    if !std::path::Path::new(&config_file).exists() {
        println!(
            "Canot find a valid config file ({})",
            config_file.underline().blue()
        );
        return Ok(());
    }

    let config = Config::load(&config_file).expect("Could not read config file");

    println!(
        "{} {}",
        "Loaded config file".dimmed(),
        config_file.dimmed().underline()
    );

    match args.action {
        Some(Action::Clean) => {
            for mut workspace in config.workspaces {
                let target = config.target.clone();

                if let Err(e) = workspace.load_name_if_possible() {
                    log::error("An error has occurred:", e);
                }

                if let Ok(files) = std::fs::read_dir(workspace.path) {
                    for file in files {
                        match file {
                            Ok(f) => {
                                if let Some(filename) = f.path().file_name().unwrap().to_str() {
                                    if f.path().exists() && f.path().is_file() && is_yaml(filename)
                                    {
                                        utils::remove_file(&f.path(), &target, &workspace.name)?;
                                        println!(
                                            "Removing {}",
                                            utils::target_filename(
                                                &f.path(),
                                                &target,
                                                &workspace.name
                                            )
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
            }
        }
        None | Some(Action::Copy) => {
            if !args.watch {
                let mut skipped = 0;

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
                    if let Ok(content) = fs::read_dir(&workspace.path) {
                        for f in content {
                            match f {
                                Ok(path) => {
                                    if let Some(filename) =
                                        path.path().file_name().unwrap().to_str()
                                    {
                                        if path.path().is_file() && is_yaml(filename) {
                                            utils::copy_file(
                                                &path.path(),
                                                &target,
                                                &workspace.name,
                                            )?
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
                }

                if skipped > 0 {
                    println!("Skipped {} files.", skipped);
                }

                return Ok(());
            }

            if args.watch {
                let handle = std::thread::spawn(move || {
                    if let Err(err) = watchers::watch_config(&config_file) {
                        log::error("An error has occurred:", err)
                    }
                });

                for mut workspace in config.workspaces {
                    let target = config.target.clone();

                    match workspace.load_name_if_possible() {
                        Ok(_) => {}
                        Err(e) => log::error("An error has occurred:", e),
                    }

                    std::thread::spawn(move || {
                        if let Err(err) = watchers::watch_sync(workspace, &target) {
                            log::error("Something went wrong:", err)
                        }
                    });
                }

                handle.join().unwrap();
            }
        }
    }

    Ok(())
}
