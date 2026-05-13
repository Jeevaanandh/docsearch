use notify::{Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::sync::{Arc, Mutex};
use std::{path::Path, sync::mpsc};

use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

fn watcherfn(rx: mpsc::Receiver<Result<Event>>) -> Result<()> {
    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.

    // Block forever, printing out events as they come in
    //
    //
    // So,
    // poll the events Eg, if the time difference between current and last event <= 10ms, then both
    // the events can be polled
    //
    // The type of event doesnt matter ----- look at the file name, check if it exists, if it does,
    // embed, if it doesnt, delete it.
    //

    let mut last_run = Instant::now();
    for res in rx {
        match res {
            Ok(event) => {
                let event_instant = Instant::now();

                let elapsed = last_run.elapsed();

                if elapsed > Duration::from_secs(1) {
                    let path = match event.paths[0].parent() {
                        Some(p) => p.to_str().unwrap().to_string(),

                        None => {
                            return Ok(());
                        }
                    };

                    println!("Embedding Started for {:?}", event.paths[0]);

                    let exe = std::env::current_exe().unwrap();

                    let output = Command::new(exe).args(["sync", &path]).status().unwrap();

                    if output.success() {
                        println!("Embedding successful");
                    } else {
                        println!("Embeddings failed");
                    }

                    last_run = event_instant;
                }
            }

            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn add_watch_listener(watcher: Arc<Mutex<RecommendedWatcher>>) -> notify::Result<()> {
    let socket_path = "/tmp/server.sock";

    let _ = std::fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;

    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)?;
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut watcher = watcher.lock().unwrap();

        watcher.watch(Path::new(message.trim()), RecursiveMode::Recursive)?;

        //Add the path to the watchlist
    }

    Ok(())
}
pub fn start_watch() -> Result<()> {
    let (tx, rx) = mpsc::channel::<Result<Event>>();

    let watcher = notify::recommended_watcher(move |res| {
        tx.send(res).unwrap();
    })?;

    let watcher = Arc::new(Mutex::new(watcher));

    let watcher_clone = watcher.clone();

    let watcher_handle = thread::spawn(move || {
        watcherfn(rx);
    });

    let listener_handle = thread::spawn(move || {
        add_watch_listener(watcher_clone);
    });

    watcher_handle.join().unwrap();
    listener_handle.join().unwrap();

    Ok(())
}
