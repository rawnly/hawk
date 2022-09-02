//! src/main.rs
use notify::event::{ModifyKind, RenameMode, DataChange, CreateKind};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::path::Path;
use std::fs;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt
};

use file_watcher::config::{Config, Workspace};


fn main() -> notify::Result<()> {
    let default_config_file: String = "config.yaml".to_string();
    let config_file: String = std::env::args().nth(1).unwrap_or(default_config_file);
    let config = Config::load(&config_file).expect("Unable to parse configuration.");

    let handle = std::thread::spawn(move || {
        if let Err(err) = watch_config(&config_file) {
            println!("An erro has occurred: {:?}", err);
        }
    });

    for workspace in config.workspaces {
        let target = config.target.clone();

        std::thread::spawn(move || {
            if let Err(err) = sync_watch(workspace, &target) {
                println!("An error has occurred: {:?}", err);
            }
        });
    }

    handle.join().unwrap();
    // futures::executor::block_on(async {
    //     for workspace in config.workspaces {
    //         if let Err(err) = watch(workspace, &config.target).await {
    //             println!("An error has occurred: {:?}", err);
    //         }
    //     }
    // });

    Ok(())
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(move |res| {
        futures::executor::block_on(async {
            tx.send(res).await.unwrap();
        })
    }, notify::Config::default())?;

    Ok((watcher, rx))
}

fn watch_config(path: &str) -> notify::Result<()> {
    println!("watching config for changes...");
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    watcher.watch(Path::new(path).as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = event.kind {
                    println!("Changes detected in the config file. Please restart")
                }
            },
            Err(err) => println!("error watching config file: {:?}", err)
        }
    }

    Ok(())
}

fn sync_watch(workspace: Workspace, target: &str) -> notify::Result<()> {
    println!("Watching: {} for {}", workspace.path, workspace.name);

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    let path = Path::new(&workspace.path);

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if event.paths.first().is_none() {
                    println!("no paths.");
                    continue
                }

                let path = event.paths.first().unwrap();

                if path.is_dir() {
                    continue
                }

                match event.kind {
                    EventKind::Remove(_) => remove_file(path, target, &workspace.name)?,
                    EventKind::Create(CreateKind::File) |
                    EventKind::Modify(ModifyKind::Data(DataChange::Content)) => copy_file(path, target, &workspace.name)?,
                    EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                        let mut iter = event.paths.iter();
                        let from = iter.next().unwrap();
                        let to = iter.next().unwrap();

                        copy_file(to, target, &workspace.name)?;
                        remove_file(from, target, &workspace.name)?;
                    },
                    EventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
                        println!("[WARNING] - Renaming is not supported yet! Please see https://github.com/notify-rs/notify/issues/261");
                    }
                    _ => continue
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

async fn watch(workspace: Workspace, target: &str) -> notify::Result<()> {
    println!("Watching: {} for {}", workspace.path, workspace.name);

    let (mut watcher, mut rx) = async_watcher()?;
    let path = Path::new(&workspace.path);

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if event.paths.first().is_none() {
                    println!("no paths.");
                    continue
                }

                let path = event.paths.first().unwrap();

                if path.is_dir() {
                    continue
                }

                match event.kind {
                    EventKind::Remove(_) => remove_file(path, target, &workspace.name)?,
                    EventKind::Create(CreateKind::File) |
                    EventKind::Modify(ModifyKind::Data(DataChange::Content)) => copy_file(path, target, &workspace.name)?,
                    EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                        let mut iter = event.paths.iter();
                        let from = iter.next().unwrap();
                        let to = iter.next().unwrap();

                        copy_file(to, target, &workspace.name)?;
                        remove_file(from, target, &workspace.name)?;
                    },
                    EventKind::Modify(ModifyKind::Name(RenameMode::Any)) => {
                        println!("[WARNING] - Renaming is not supported yet! Please see https://github.com/notify-rs/notify/issues/261");
                    }
                    _ => continue
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn copy_file(source: &Path, target_dir: &str , scope: &str) -> std::io::Result<()> {
    let filename = target_filename(source, target_dir, scope);
    to_void_result(fs::copy(source, filename))
}

fn remove_file(source: &Path, target: &str, scope: &str) -> std::io::Result<()> {
    let filename = target_filename(source, target, scope);

    if Path::new(&filename).exists() {
        fs::remove_file(filename)?
    }

    Ok(())
}

fn target_filename(source: &Path, target: &str, scope: &str) -> String {
    let name = source
        .file_name().unwrap()
        .to_str().unwrap();

    format!("{}/{}--{}", target, scope, name)
}

fn to_void_result<T>(r: std::io::Result<T>) -> std::io::Result<()> {
    match r {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}
