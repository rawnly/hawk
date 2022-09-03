//! src/main.rs
use hawk::log;
use hawk::utils;
use hawk::watchers;
use hawk::models::config::Config;
use std::fs;

use clap::Parser;
use colored::*;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, value_parser, default_value_t = false)]
    watch: bool,

    #[clap(short, long, value_parser)]
    config: Option<String>,
}

fn is_yaml(path: &str) -> bool {
    path.ends_with("yml") || path.ends_with("yaml")
}

fn main() -> notify::Result<()> {
    let args = Args::parse();

    let default_config_file: String = "hawk-config.yaml".to_string();
    let config_file: String = args.config.unwrap_or(default_config_file);

    if !std::path::Path::new(&config_file).exists() {
        println!("Canot find a valid config file ({})", config_file.underline().blue());
        return Ok(());
    }

    let config = Config::load(&config_file).expect("Could not read config file");

    println!(
        "{} {}",
        "Loaded config file".dimmed(),
        config_file.dimmed().underline()
    );

    if !args.watch {
        let mut skipped = 0;

        for mut workspace in config.workspaces {
            let target = config.target.clone();

            match workspace.load_name_if_possible() {
                Ok(_) => {}
                Err(e) => log::error("An error has occurred:", e),
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
                            if let Some(filename) = path.path().file_name().unwrap().to_str() {
                                if path.path().is_file() && is_yaml(filename) {
                                    utils::copy_file(&path.path(), &target, &workspace.name)?
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

    Ok(())
}
