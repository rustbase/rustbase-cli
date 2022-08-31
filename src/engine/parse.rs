use crate::engine::Request;
use colored::Colorize;
use pest::iterators::Pair;

use super::{Response, Rustbase};
use bson::{Bson, Document};
use pest::Parser;

#[derive(Parser)]
#[grammar = "engine/grammar/rustbase.pest"]
struct RustbaseParser;

fn parser_to_bson(pair: Pair<Rule>) -> Bson {
    match pair.as_rule() {
        Rule::object => {
            let mut doc = Document::new();
            for pair in pair.into_inner() {
                let mut inner_rules = pair.into_inner();
                let key = inner_rules
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_string()
                    .replace('"', ""); // bro, this ident is ugly lol

                let value = parser_to_bson(inner_rules.next().unwrap());
                doc.insert(key, value);
            }
            Bson::Document(doc)
        }
        Rule::array => {
            let mut arr = Vec::new();
            for pair in pair.into_inner() {
                arr.push(parser_to_bson(pair));
            }
            Bson::Array(arr)
        }
        Rule::string => Bson::String(
            pair.into_inner()
                .next()
                .unwrap()
                .as_str()
                .to_string()
                .replace('"', ""),
        ),
        Rule::number => Bson::Int64(pair.as_str().parse().unwrap()),
        Rule::boolean => Bson::Boolean(pair.as_str().parse().unwrap()),
        Rule::null => Bson::Null,
        Rule::json
        | Rule::EOI
        | Rule::pair
        | Rule::value
        | Rule::inner
        | Rule::char
        | Rule::WHITESPACE
        | Rule::insert
        | Rule::update
        | Rule::methods
        | Rule::delete
        | Rule::get => unreachable!(),
    }
}

pub async fn parse(input: String, mut client: Rustbase) {
    let pairs = match RustbaseParser::parse(Rule::methods, &input) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("{} {}", "[Error]".red(), e);
            return;
        }
    };

    for pair in pairs {
        match pair.as_rule() {
            Rule::insert => {
                let key = pair.clone().into_inner().nth(1).unwrap().as_str();
                let value = pair.into_inner().next().unwrap();

                let doc = parser_to_bson(value);

                if let Some(doc) = doc.as_document() {
                    match client
                        .request(Request::Insert(key.to_string(), doc.clone()))
                        .await
                    {
                        Ok(res) => match res {
                            Response::Insert(_) => {
                                println!("{} ok", "[Success]".green());
                            }

                            _ => {
                                println!("{} failed", "[Error]".red());
                            }
                        },
                        Err(e) => {
                            println!("{} {}", "[Error]".red(), e);
                        }
                    }
                } else {
                    println!("{} Is not a document", "[Error]".red());
                }
            }

            Rule::update => {
                let key = pair.clone().into_inner().nth(1).unwrap().as_str();
                let value = pair.into_inner().next().unwrap();

                let doc = parser_to_bson(value);

                if let Some(doc) = doc.as_document() {
                    match client
                        .request(Request::Update(key.to_string(), doc.clone()))
                        .await
                    {
                        Ok(res) => match res {
                            Response::Update(_) => {
                                println!("{} ok", "[Success]".green());
                            }

                            _ => {
                                println!("{} failed", "[Error]".red());
                            }
                        },
                        Err(e) => {
                            println!("{} {}", "[Error]".red(), e);
                        }
                    }
                } else {
                    println!("{} Is not a document", "[Error]".red());
                }
            }

            Rule::delete => {
                let key = pair.clone().into_inner().next().unwrap().as_str();

                match client.request(Request::Delete(key.to_string())).await {
                    Ok(_) => {
                        println!("{} ok", "[Success]".green());
                    }
                    Err(e) => {
                        eprintln!("{} {}", "[Error]".red(), e);
                    }
                }
            }

            Rule::get => {
                let key = pair.clone().into_inner().next().unwrap().as_str();

                match client.request(Request::Get(key.to_string())).await {
                    Ok(res) => match res {
                        Response::Get(doc) => {
                            println!("{}", bson::to_bson(&doc).unwrap());
                        }

                        _ => {
                            unreachable!()
                        }
                    },
                    Err(e) => {
                        eprintln!("{} {}", "[Error]".red(), e);
                    }
                }
            }

            _ => unreachable!(),
        }
    }
}
