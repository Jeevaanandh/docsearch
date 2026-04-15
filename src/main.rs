use clap::{Parser, Subcommand};

mod app;
mod file_test;
mod open_file;
mod pdf_test;
mod ppt_test;
mod repository;
mod search;
mod watcher;

use crate::repository::db::db_init;
use app::run_app;
use file_test::{check_diff, parse_directory, parse_directory2};
use search::search;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

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
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Search { prompt: String },

    Init,

    Sync,
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
        Command::Search { prompt } => {
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

        Command::Init => {
            let pool = match db_init().await {
                Ok(p) => p,

                Err(_) => {
                    println!("ERROR");
                    return;
                }
            };
            parse_directory(&pool).await;
        }

        Command::Sync => {
            let pool = match db_init().await {
                Ok(p) => p,

                Err(_) => {
                    println!("ERROR");
                    return;
                }
            };

            check_diff(&pool).await;
        }
    }
}
