use clap::Parser;

mod engine;
mod utils;

use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

/// A CLI for Rustbase Database Server
#[derive(clap_derive::Parser)]
#[clap(author, about, long_about = None)]
struct Args {
    /// Host address to connect to Rustbase Database Server
    #[clap(short, long, default_value = "localhost")]
    host: String,
    /// Port number to connect to Rustbase Database Server
    #[clap(short, long, default_value = "23561")]
    port: String,
    /// Use TLS for connection
    #[clap(long, action)]
    tls: bool,
    /// CA file path (only for TLS)
    #[clap(long, default_value = "")]
    ca_file: String,

    #[clap(subcommand)]
    commands: Option<Commands>,
}

#[derive(clap_derive::Subcommand, PartialEq)]
enum Commands {
    #[clap(about = "Upgrade Rustbase and Rustbase CLI")]
    Upgrade,
    #[clap(about = "Clean repl history")]
    Clean,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let repl_path = utils::get_current_path().join("repl.history");

    match args.commands {
        Some(Commands::Upgrade) => {
            println!("Not implemented yet");
            return Ok(());
        }
        Some(Commands::Clean) => {
            println!("{} Cleaning repl history", "[Command]".cyan());
            if repl_path.exists() {
                std::fs::remove_file(&repl_path).unwrap();
                println!("{} Ok", "[Success]".green());
            } else {
                println!(
                    "{} Repl history file could not be found.",
                    "[Warning]".yellow()
                );
            }

            return Ok(());
        }

        None => {}
    }

    let exe_name = std::env::current_exe().unwrap();
    let exe_name = exe_name.file_name().unwrap().to_str().unwrap();

    println!("{}", "Welcome to Rustbase CLI!".bold());
    println!("Current version: v{}", env!("CARGO_PKG_VERSION").cyan());
    println!();
    println!(
        "Trying to connect to rustbase://{}:{}",
        args.host, args.port
    );
    println!(
        "To change the server address, use: {} --host {} --port {}",
        exe_name.green(),
        "<host>".to_string().green(),
        "<port>".to_string().green(),
    );
    println!("Press Ctrl+C to exit.");

    let mut rl = Editor::<()>::new()?;

    rl.load_history(repl_path.to_str().unwrap()).ok();
    println!();

    let mut database = rl.readline("Database: ")?;

    loop {
        if database == "" {
            database = rl.readline("Database: ")?;
            continue;
        } else {
            break;
        }
    }

    let mut client = if args.tls {
        engine::Rustbase::connect_tls(args.host, args.port, database.clone(), args.ca_file).await
    } else {
        engine::Rustbase::connect(args.host, args.port, database.clone()).await
    };

    loop {
        let readline = rl.readline(format!("{}> ", database).as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                if line == "exit" {
                    println!("bye");
                    break;
                } else if line.starts_with("use") {
                    let prompted_database = line.split(' ').collect::<Vec<&str>>()[1].to_string();
                    client.database = prompted_database.clone();
                    database = prompted_database;
                    continue;
                }

                if !line.is_empty() {
                    if args.tls {
                        client.request_tls(engine::Request::Query(line)).await;
                    } else {
                        client.request(engine::Request::Query(line)).await;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("bye");
                rl.save_history(repl_path.to_str().unwrap()).ok();

                break;
            }
            Err(ReadlineError::Eof) => {
                println!("bye");
                rl.save_history(repl_path.to_str().unwrap()).ok();
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(repl_path.to_str().unwrap())
}
