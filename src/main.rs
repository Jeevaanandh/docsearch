use clap::{Parser, Subcommand};

mod app;
mod daemon;
mod embed;
mod file_test;
mod open_file;
mod pdf_test;
mod ppt_test;
mod repository;
mod search;
mod search_new;
mod watcher;

use crate::repository::db::{create_watch, db_init};
use app::run_app;
use daemon::{get_daemon, start_daemon, stop_daemon};
use file_test::{check_diff, parse_directory, parse_directory2};

use search_new::search;
use std::env;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use watcher::start_watch;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use std::fs;

use std::io;
use std::process::Command;

pub struct App {
    files: Vec<String>,
    filepaths: Vec<String>,
    selected: usize,
}

impl App {
    fn new(results: (Vec<String>, Vec<String>)) -> App {
        App {
            files: results.0,
            filepaths: results.1,
            selected: 0,
        }
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.files.len();
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.files.len() - 1;
        }
    }

    fn get_selected_option(&self) -> &str {
        &self.filepaths[self.selected]
    }
}

#[derive(Parser)]
#[command(name = "docsearch", about = "Document Search")]
struct Args {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Search { prompt: String },

    Init,

    Sync { path: Option<String> },

    Begin, //This is to start the watcher for the daemon

    Start, // This is to start the watcher using docsearch.

    Add, //This is to add a directory to the watchlist,

    Stop,
}

fn setup_app(app: &mut App) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = run {
        println!("Error: {:?}", err);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Cmd::Search { prompt } => {
            let pool = match db_init().await {
                Ok(p) => p,

                Err(_) => {
                    println!("ERROR");

                    return;
                }
            };

            let result = search(&prompt, &pool).await;
            let result2 = result.clone();

            let files = result.0;
            let filepaths = result.1;

            if files.len() == 0 || filepaths.len() == 0 {
                println!("No Results Found!");
                return;
            }

            let mut app: App = App::new(result2);
            match setup_app(&mut app) {
                Ok(_) => {
                    return;
                }

                Err(e) => {
                    println!("Error in App {}", e);
                    return;
                }
            };
        }

        Cmd::Init => {
            let pool = match db_init().await {
                Ok(p) => p,

                Err(_) => {
                    println!("ERROR");
                    return;
                }
            };
            parse_directory(&pool).await;
        }

        Cmd::Sync { path } => {
            let pool = match db_init().await {
                Ok(p) => p,

                Err(_) => {
                    println!("ERROR");
                    return;
                }
            };

            match path {
                Some(p) => {
                    check_diff(&pool, &p).await;
                }

                None => {
                    check_diff(&pool, "").await;
                }
            }
        }

        Cmd::Add => {
            let current_dir = env::current_dir().unwrap().to_str().unwrap().to_string();

            let mut stream = match UnixStream::connect("/tmp/server.sock") {
                Ok(r) => r,

                Err(e) => {
                    println!("Error: {}", e);
                    return;
                }
            };
            match stream.write_all(current_dir.as_bytes()) {
                Ok(r) => {
                    println!("Watch Added Successfully");
                }
                Err(e) => {
                    println!("Error: {}", e);
                    return;
                }
            };

            let mut res = String::new();

            match stream.read_to_string(&mut res) {
                Ok(r) => {}

                Err(e) => {
                    println!("Error in reading from the stream");
                    return;
                }
            };

            println!("{}", res);
        }

        Cmd::Begin => {
            let pool = match create_watch().await {
                Ok(p) => p,

                Err(_) => {
                    println!("Error in creating the watch table");
                    return;
                }
            };

            match start_watch(&pool).await {
                Ok(_) => {}

                Err(_) => {
                    println!("Error in starting the daemon");
                }
            };
        }

        Cmd::Start => {
            let home = env::home_dir().unwrap().to_str().unwrap().to_string();
            let daemonpath = {
                if cfg!(target_os = "linux") {
                    format!("{}/docsearch.service", home)
                } else if cfg!(target_os = "macos") {
                    format!("{}/com.docsearch.plist", home)
                } else {
                    "".to_string()
                }
            };
            let daemon_str = get_daemon();

            match fs::write(&daemonpath, daemon_str) {
                Ok(_) => {}

                Err(_) => {
                    println!("Error in writing the plist file");
                    return;
                }
            }

            start_daemon(&daemonpath);
        }

        Cmd::Stop => {
            let home = env::home_dir().unwrap().to_str().unwrap().to_string();
            let daemonpath = {
                if cfg!(target_os = "linux") {
                    format!("{}/docsearch.service", home)
                } else if cfg!(target_os = "macos") {
                    format!("{}/com.docsearch.plist", home)
                } else {
                    "".to_string()
                }
            };

            stop_daemon(&daemonpath);
        }
    }
}
