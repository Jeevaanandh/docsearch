use clap::{Parser,Subcommand};
mod pdf_test;
mod file_test;
mod repository;

use pdf_test::extract_pdf;
use file_test::parse_directory;
use crate::repository::db::db_init;

#[derive(Parser)]
#[command(name= "docsearch", about= "Document Search")]
struct Args{
    #[command(subcommand)]
    command: Command,

}

#[derive(Subcommand)]
enum Command{
    Search{
        prompt:String
    },

    Init,

    Test,

    TestDir
}

#[tokio::main]
async fn main() {
    let args= Args::parse();

    match args.command{
        Command::Search{prompt} =>{
            println!("Prompt: {}", prompt);

        }

        Command::Init =>{
            println!("init was run");
        }


        Command::Test => {
            let pool = match db_init().await{
                        Ok(p) => p,

                        Err(_) =>{
                            println!("ERROR");
                            return
                        }

                

                    };

            match extract_pdf("testFile.pdf", &pool).await{
                Ok(_) => {
                    println!("Successfully extracted the PDF");
                }

                Err(e) =>{
                    println!("Error occured {}", e);
                }
            };

        }


        Command::TestDir => {
                    let pool = match db_init().await{
                        Ok(p) => p,

                        Err(_) =>{
                            println!("ERROR");
                            return
                        }

                

                    };
            parse_directory(&pool).await;
        }
    }
}
