use notify::{Event, RecursiveMode, Result, Watcher};
use std::{path::Path, sync::mpsc};

pub fn add_watch() {}

pub fn watch() -> Result<()> {
    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    // Block forever, printing out events as they come in
    for res in rx {
        match res {
            Ok(event) => println!("event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
