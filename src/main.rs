use clap::{Parser, Subcommand};
mod file_test;
mod pdf_test;
mod repository;
mod search;

use crate::repository::db::db_init;
use file_test::{check_diff, parse_directory};
use pdf_test::extract_pdf;
use search::search;

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

    Test,

    TestDir,

    Sync,
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

            search(&prompt, &pool).await;
        }

        Command::Init => {
            println!("init was run");
        }

        Command::Test => {
            let pool = match db_init().await {
                Ok(p) => p,

                Err(_) => {
                    println!("ERROR");

                    return;
                }
            };

            match extract_pdf("testFile.pdf", &pool).await {
                Ok(_) => {
                    println!("Successfully extracted the PDF");
                }

                Err(e) => {
                    println!("Error occured {}", e);
                }
            };
        }

        Command::TestDir => {
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
