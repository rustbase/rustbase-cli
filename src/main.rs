use clap::Parser;

mod engine;
mod utils;

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

    #[clap(subcommand)]
    commands: Option<Commands>,
}

#[derive(clap_derive::Subcommand, PartialEq)]
enum Commands {
    Upgrade,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.commands == Some(Commands::Upgrade) {
        println!("Not implemented yet");
    }

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

    rl.load_history("repl.history").ok();

    let mut database = rl.readline("Database: ")?;

    let mut client = engine::Rustbase::connect(args.host, args.port, database.clone()).await;

    loop {
        let readline = rl.readline(format!("{}> ", database).as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                if line == "exit" {
                    println!("bye");
                    break;
                } else if line.trim() == "!database" {
                    let prompted_database = rl.readline("Database: ")?;
                    client.database = prompted_database.clone();
                    database = prompted_database;
                    continue;
                }

                if !line.is_empty() {
                    client.request(engine::Request::Query(line)).await;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("bye");
                rl.save_history("repl.history").ok();

                break;
            }
            Err(ReadlineError::Eof) => {
                println!("bye");
                rl.save_history("repl.history").ok();
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(
        utils::get_current_path()
            .join("repl.history")
            .to_str()
            .unwrap(),
    )
}
