use rustbase::rustbase_client::RustbaseClient;
use rustbase::{Key, KeyValue};
use tonic::transport::Channel;
use tonic::Code;

pub mod parse;

pub mod rustbase {
    tonic::include_proto!("rustbase");
}

#[derive(Debug)]
pub enum Request {
    Get(String),
    Insert(String, bson::Document),
    Update(String, bson::Document),
    Delete(String),
}

#[derive(Debug)]
pub enum Response {
    Get(bson::Document),
    Insert(()),
    Update(()),
    Delete(()),
}

type Result<T> = std::result::Result<T, Error>;

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
}

impl Rustbase {
    pub async fn connect(host: String, port: String) -> Rustbase {
        match RustbaseClient::connect(format!("http://{}:{}", host.clone(), port.clone())).await {
            Ok(client) => {
                println!("Connected to rustbase://{}:{}", host, port);
                Rustbase { client }
            }
            Err(e) => {
                eprintln!("Cannot connect the server: {}", e);

                std::process::exit(1);
            }
        }
    }

    pub async fn request(&mut self, request: Request) -> Result<Response> {
        match request {
            Request::Get(key) => {
                let response = self.client.get(Key { key }).await;

                match response {
                    Ok(res) => Ok(Response::Get(
                        bson::from_slice(&res.into_inner().bson).unwrap(),
                    )),
                    Err(e) => Err(Error { status: e }),
                }
            }
            Request::Insert(key, doc) => {
                let response = self
                    .client
                    .insert(KeyValue {
                        key,
                        value: bson::to_vec(&doc).unwrap(),
                    })
                    .await;

                match response {
                    Ok(_) => Ok(Response::Insert(())),
                    Err(e) => Err(Error { status: e }),
                }
            }
            Request::Update(key, doc) => {
                let response = self
                    .client
                    .update(KeyValue {
                        key,
                        value: bson::to_vec(&doc).unwrap(),
                    })
                    .await;
                match response {
                    Ok(_) => Ok(Response::Update(())),
                    Err(e) => Err(Error { status: e }),
                }
            }
            Request::Delete(key) => {
                let response = self.client.delete(Key { key }).await;
                match response {
                    Ok(_) => Ok(Response::Delete(())),
                    Err(e) => Err(Error { status: e }),
                }
            }
        }
    }
}
