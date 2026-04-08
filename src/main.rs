use clap::{Parser, Subcommand};
mod app;
mod file_test;
mod open_file;
mod pdf_test;
mod ppt_test;
mod repository;
mod search;

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
    options: Vec<String>,
    selected: usize,
}

impl App {
    fn new(results: Vec<String>) -> App {
        App {
            options: results,
            selected: 0,
        }
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.options.len() - 1;
        }
    }

    fn get_selected_option(&self) -> &str {
        &self.options[self.selected]
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

            if result.len() == 0 {
                println!("No Results Found!");
                return;
            }

            let mut app: App = App::new(result);
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
            parse_directory2(&pool).await;
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
