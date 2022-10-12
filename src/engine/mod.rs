use core::panic;

use colored::Colorize;
use rustbase::rustbase_client::RustbaseClient;
use rustbase::{QueryMessage, QueryResult, QueryResultType};
use tonic::transport::Channel;
use tonic::Code;

pub mod rustbase {
    tonic::include_proto!("rustbase");
}

#[derive(Debug)]
pub enum Request {
    Query(String),
}

#[derive(Debug)]
pub struct Error {
    status: tonic::Status,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.status.code() {
            Code::NotFound => write!(f, "Key not found"),

            _ => write!(f, "{}", self.status),
        }
    }
}

#[derive(Clone)]
pub struct Rustbase {
    pub client: RustbaseClient<Channel>,
    pub database: String,
}

impl Rustbase {
    pub async fn connect(host: String, port: String, database: String) -> Rustbase {
        match RustbaseClient::connect(format!("http://{}:{}", host.clone(), port.clone())).await {
            Ok(client) => Rustbase { client, database },
            Err(e) => {
                eprintln!("Cannot connect the server: {}", e);

                std::process::exit(1);
            }
        }
    }

    pub async fn request(&mut self, request: Request) {
        match request {
            Request::Query(query) => {
                let query = QueryMessage {
                    query,
                    database: self.database.clone(),
                };

                match self.client.query(query.clone()).await {
                    Ok(response) => {
                        let response = response.into_inner();

                        match parse_i32_to_enum(response.result_type) {
                            QueryResultType::Ok => {
                                if let Some(result) = response.bson {
                                    let doc = bson::from_slice::<bson::Document>(&result).unwrap();

                                    if let Some(value) = doc.get("_l") {
                                        println!("{}", value);
                                    } else {
                                        println!("{}", doc);
                                    }
                                } else {
                                    println!("{} OK", "[Success]".green());
                                }
                            }
                            QueryResultType::Error => {
                                println!(
                                    "{} {}",
                                    "[Error]".red().bold(),
                                    response.error_message.unwrap()
                                );
                            }
                            QueryResultType::NotFound => {
                                print!("{} Not found: ", "[Error]".red().bold());
                                if let Some(msg) = response.error_message {
                                    println!("{}", msg);
                                }
                            }
                            QueryResultType::SyntaxError => {
                                println!("[Error] Syntax: \n{}", response.error_message.unwrap());
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
    }
}

fn parse_i32_to_enum(num: i32) -> QueryResultType {
    match num {
        0 => QueryResultType::Ok,
        1 => QueryResultType::NotFound,
        2 => QueryResultType::Error,
        3 => QueryResultType::SyntaxError,
        _ => panic!("Invalid number"),
    }
}
