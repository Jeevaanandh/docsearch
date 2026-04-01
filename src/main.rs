use clap::{Parser,Subcommand};
use std::thread;
mod pdf_test;
mod file_test;

use pdf_test::extract_pdf;
use file_test::parse_directory;

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


fn main() {
    let args= Args::parse();

    match args.command{
        Command::Search{prompt} =>{
            println!("Prompt: {}", prompt);

            let handle = thread::spawn(|| {
                println!("Hello from Thread!!");
            });

            handle.join().unwrap();
        }

        Command::Init =>{
            println!("init was run");
        }


        Command::Test => {
            match extract_pdf("testFile.pdf"){
                Ok(_) => {
                    println!("Successfully extracted the PDF");
                }

                Err(_) =>{
                    println!("Error occured");
                }
            };

        }


        Command::TestDir => {
            parse_directory();
        }
    }
}
