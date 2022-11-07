use colored::Colorize;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub enum Request {
    Query(String),
}

const BUFFER_SIZE: usize = 1024 * 1024 * 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub message: Option<String>,
    pub status: Status,
    pub body: Option<bson::Bson>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Error,
    DatabaseNotFound,
    KeyNotExists,
    KeyAlreadyExists,
    SyntaxError,
    InvalidQuery,
    InvalidBody,
}

pub struct Rustbase {
    pub client: TcpStream,
    pub database: String,
}

impl Rustbase {
    pub async fn connect(host: String, port: String, database: String) -> Rustbase {
        let client = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .unwrap();

        Rustbase { client, database }
    }

    pub async fn request(&mut self, request: Request) {
        match request {
            Request::Query(query) => {
                let doc = bson::doc! {
                    "body": {
                        "query": query,
                        "database": self.database.clone(),
                    },
                };

                self.client
                    .write_all(&bson::to_vec(&doc).unwrap())
                    .await
                    .unwrap();

                let mut buf = vec![0; BUFFER_SIZE];
                let n = self.client.read(&mut buf).await.unwrap();

                let doc: Response = bson::from_slice(&buf[..n]).unwrap();

                match doc.status {
                    Status::Ok => {
                        if doc.body.is_some() {
                            println!("{}", doc.body.unwrap());
                        } else {
                            println!("{} Ok", "[Success]".green());
                        }
                    }

                    Status::SyntaxError => {
                        println!("{}", "[Error]".red());
                        println!("{}", doc.message.unwrap());
                    }

                    _ => {
                        println!("{} {}", "[Error]".red(), status_string(doc.status));
                    }
                }
            }
        }
    }
}

pub fn status_string(status: Status) -> String {
    match status {
        Status::Ok => "Ok".to_string(),
        Status::Error => "Error".to_string(),
        Status::DatabaseNotFound => "DatabaseNotFound".to_string(),
        Status::KeyNotExists => "KeyNotExists".to_string(),
        Status::KeyAlreadyExists => "KeyAlreadyExists".to_string(),
        Status::SyntaxError => "SyntaxError".to_string(),
        Status::InvalidQuery => "InvalidQuery".to_string(),
        Status::InvalidBody => "InvalidBody".to_string(),
    }
}
