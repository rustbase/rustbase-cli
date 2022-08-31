#[macro_use]
extern crate pest_derive;

use clap::Parser;

mod engine;

use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

#[derive(clap_derive::Parser)]
#[clap(version)]
struct Args {
    #[clap(short, long, default_value = "localhost")]
    host: String,
    #[clap(short, long, default_value = "23561")]
    port: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Welcome to Rustbase Shell!");
    println!("Press Ctrl+C to exit.");
    println!();
    println!(
        "Trying to connect to rustbase://{}:{}",
        args.host, args.port
    );
    println!(
        "To change the server address, use: {} --host {} --port {}",
        "rustbase".to_string().green(),
        "<host>".to_string().green(),
        "<port>".to_string().green(),
    );

    let mut rl = Editor::<()>::new()?;

    if rl.load_history("repl.history").is_err() {
        println!("No previous history.");
    }

    let client = engine::Rustbase::connect(args.host, args.port).await;

    loop {
        let readline = rl.readline("RUSTBASE> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                if line == "exit" {
                    println!("bye");
                    break;
                }

                if !line.is_empty() {
                    engine::parse::parse(line, client.clone()).await;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("bye");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("bye");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("repl.history")
}
