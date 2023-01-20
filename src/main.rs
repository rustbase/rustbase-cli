use clap::Parser;

mod engine;
mod utils;

use colored::Colorize;

use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::{ColorMode, Editor, Result};
use rustyline_derive::{Completer, Helper, Hinter, Validator};

use std::borrow::Cow::{self, Borrowed, Owned};

use engine::{AuthConfig, RustbaseConfig, TlsConfig};

#[derive(Completer, Helper, Hinter, Validator)]
struct MaskingHighlighter {
    masking: bool,
}

impl Highlighter for MaskingHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        use unicode_width::UnicodeWidthStr;
        if self.masking {
            Owned("*".repeat(line.width()))
        } else {
            Borrowed(line)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        self.masking
    }
}

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

    /// Use authentication for connection
    #[clap(long, short = 'a')]
    use_auth: bool,

    #[clap(subcommand)]
    commands: Option<Commands>,
}

fn prompt<T>(prompt: &str, editor: &mut Editor<T>) -> Result<String>
where
    T: rustyline::Helper,
{
    let result = loop {
        let prompt_result = editor.readline(prompt)?;

        if !prompt_result.is_empty() {
            break prompt_result;
        }
    };

    Ok(result)
}

#[derive(clap_derive::Subcommand, PartialEq)]
enum Commands {
    #[clap(about = "Update Rustbase CLI")]
    Update,
    #[clap(about = "Clean repl history")]
    Clean,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let repl_path = utils::get_current_path().join("repl.history");

    match args.commands {
        Some(Commands::Update) => {
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

    let h = MaskingHighlighter { masking: false };
    let mut rl = Editor::new()?;
    rl.set_helper(Some(h));

    rl.load_history(repl_path.to_str().unwrap()).ok();
    println!();

    let mut database = prompt("Database: ", &mut rl)?;

    let auth_config = if args.use_auth {
        let username = prompt("Username: ", &mut rl)?;

        rl.helper_mut().expect("No helper").masking = true;
        rl.set_color_mode(ColorMode::Forced); // force masking
        rl.set_auto_add_history(false); // make sure password is not added to history

        let password = prompt("Password: ", &mut rl)?;

        rl.helper_mut().expect("No helper").masking = false;
        rl.set_color_mode(ColorMode::Disabled);
        rl.set_auto_add_history(true);

        Some(AuthConfig { username, password })
    } else {
        None
    };

    let tls_config = if args.tls {
        let ca_file = args.ca_file;

        Some(TlsConfig { ca_file })
    } else {
        None
    };

    let config = RustbaseConfig {
        database: database.clone(),
        port: args.port,
        host: args.host,
        auth: auth_config,
        tls: tls_config,
    };

    let mut client = engine::Rustbase::connect(config).await;

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
                    client.request(engine::Request::Query(line), args.tls).await;
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
