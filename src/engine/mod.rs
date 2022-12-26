use colored::Colorize;
use rustls::OwnedTrustAnchor;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::{client::TlsStream, TlsConnector};

#[derive(Debug)]
pub enum Request {
    Query(String),
}

const BUFFER_SIZE: usize = 1024 * 8;

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
    InvalidBson,
    InvalidAuth,
}

pub struct Rustbase {
    pub host: String,
    pub client: Option<TcpStream>,
    pub database: String,
    pub tls_client: Option<TlsStream<TcpStream>>,
}

impl Rustbase {
    pub async fn connect(host: String, port: String, database: String) -> Rustbase {
        let client = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .unwrap();

        Rustbase {
            client: Some(client),
            database,
            tls_client: None,
            host,
        }
    }

    pub async fn connect_tls(
        host: String,
        port: String,
        database: String,
        ca_file: String,
    ) -> Rustbase {
        if !Path::new(&ca_file).exists() {
            println!(
                "{} CA file not found: use --ca_file=<path>",
                "[Error]".red()
            );
        }

        let mut root_cert_store = rustls::RootCertStore::empty();
        let mut pem = BufReader::new(File::open(ca_file).unwrap());
        let certs = rustls_pemfile::certs(&mut pem).unwrap();
        let trust_anchors = certs.iter().map(|cert| {
            let ta = webpki::TrustAnchor::try_from_cert_der(&cert[..]).unwrap();
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        });
        root_cert_store.add_server_trust_anchors(trust_anchors);

        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));

        let domain = rustls::ServerName::try_from(host.as_str())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid dnsname"))
            .unwrap();

        let client = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .unwrap();

        let tls_client = connector.connect(domain, client).await.unwrap();

        Rustbase {
            client: None,
            database,
            tls_client: Some(tls_client),
            host,
        }
    }

    pub async fn request(&mut self, request: Request) {
        let client = self.client.as_mut().unwrap();
        match request {
            Request::Query(query) => {
                let doc = bson::doc! {
                    "body": {
                        "query": query,
                        "database": self.database.clone(),
                    },
                };

                client
                    .write_all(&bson::to_vec(&doc).unwrap())
                    .await
                    .unwrap();

                let mut buf = vec![0; BUFFER_SIZE];
                let n = client.read(&mut buf).await.unwrap();

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

    pub async fn request_tls(&mut self, request: Request) {
        let client = self.tls_client.as_mut().unwrap();
        match request {
            Request::Query(query) => {
                let doc = bson::doc! {
                    "body": {
                        "query": query,
                        "database": self.database.clone(),
                    },
                };

                client
                    .write_all(&bson::to_vec(&doc).unwrap())
                    .await
                    .unwrap();

                let mut buf = vec![0; BUFFER_SIZE];
                let n = client.read(&mut buf).await.unwrap();

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
        Status::InvalidBson => "InvalidBson".to_string(),
        Status::InvalidAuth => "InvalidAuth".to_string(),
    }
}
