//! src/main.rs
use colored::*;
use notify::event::{CreateKind, DataChange, ModifyKind, RenameMode};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

use crate::models::workspace::Workspace;
use crate::log;
use crate::utils;

pub fn watch_config(path: &str) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    watcher.watch(Path::new(path).as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = event.kind {
                    log::warn("Changes detected in the config file. Please restart")
                }
            }
            Err(err) => log::error("Something went wrong:", err),
        }
    }

    Ok(())
}

pub fn watch_sync(workspace: Workspace, target: &str) -> notify::Result<()> {
    println!(
        "[{}] {} for {}",
        "WATCH".bold().blue(),
        workspace.path.bright_yellow().bold(),
        workspace.name.bold().cyan()
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    let path = Path::new(&workspace.path);

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if event.paths.first().is_none() {
                    log::warn("No paths...");
                    continue;
                }

                let path = event.paths.first().unwrap();

                if path.is_dir() {
                    continue;
                }

                match event.kind {
                    EventKind::Remove(_) => utils::remove_file(path, target, &workspace.name)?,
                    EventKind::Create(CreateKind::File)
                    | EventKind::Modify(ModifyKind::Data(DataChange::Content)) => {
                        utils::copy_file(path, target, &workspace.name)?
                    }
                    EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                        let mut iter = event.paths.iter();
                        let from = iter.next().unwrap();
                        let to = iter.next().unwrap();

                        utils::copy_file(to, target, &workspace.name)?;
                        utils::remove_file(from, target, &workspace.name)?;
                    }
                    EventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
                        log::warn("Renaming is not supported yet! Please see https://github.com/notify-rs/notify/issues/261");
                    }
                    _ => continue,
                }
            }
            Err(e) => log::error("watch error:", e),
        }
    }

    Ok(())
}
