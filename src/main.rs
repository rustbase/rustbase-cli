use clap::Parser;
use std::fs::{remove_file, File};
use std::io::copy;

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
        // This is basically https://github.com/rustbase/rustbase-install/blob/main/install.sh
        // This only works on x64 Linux ._.
        let repo = "https://github.com/rustbase/rustbase";
        let rustbase_download = format!("{}/releases/latest/download/rustbase-linux-x64.zip", repo);
        let home = home::home_dir().unwrap();

        if systemctl::is_active("rustbase.service").unwrap() {
            println!("Stopping rustbase service...");
            systemctl::stop("rustbase.service").unwrap();
        }

        println!("Upgrading rustbase...");

        let temp_dir = std::path::Path::new(&std::env::temp_dir()).join("rustbase.zip");

        let resp = reqwest::get(rustbase_download)
            .await
            .expect("request failed");

        let bytes = resp.bytes().await.expect("failed to read bytes").to_vec();

        let mut out = File::create(temp_dir.clone()).expect("failed to create file");

        copy(&mut bytes.as_slice(), &mut out).expect("failed to copy content");

        zip::ZipArchive::new(File::open(temp_dir.clone()).unwrap())
            .unwrap()
            .extract(temp_dir.clone().parent().unwrap())
            .unwrap();

        std::fs::create_dir_all(home.join("rustbase").join("bin")).unwrap();

        std::fs::rename(
            temp_dir.clone().parent().unwrap().join("rustbase"),
            home.join("rustbase").join("bin").join("rustbase_server"),
        )
        .unwrap();

        remove_file(temp_dir.clone()).unwrap();

        if let Ok(true) = systemctl::exists("rustbase.service") {
            systemctl::restart("rustbase.service").unwrap();
        }

        println!(
            "{} Rustbase upgraded successfully!",
            "[Success]".to_string().green()
        );

        return Ok(());
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
