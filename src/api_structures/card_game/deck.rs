use pest::Parser;
use pest_derive::Parser;
use serde::de::value;
use std::{collections::HashMap, hash::Hash};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Selector {
    Current,
    Previous,
    Next,
    Random,
}

#[derive(Debug, Clone)]
pub enum MathOperation {
    Add,
    Sub,
    Div,
    Mul,
}

#[derive(Debug, Clone)]
pub enum Operation {
    GetFromTable {
        table: String,
        filter: String,
        amount: i32,
    },
    GetFromPlayers {
        selector: Selector,
    },
    GetStateFromPlayer {
        selector: Selector,
        state: String,
    },
    UpdateState {
        id: Uuid,
        state: String,
        math_operation: MathOperation,
        value: i32,
    },
    Error(String),
    RawText(String),
}

#[derive(Debug, Clone)]
pub struct Card {
    pub operations: Vec<Operation>,
    pub id: Uuid,
}

pub fn into_operation(raw: String) -> Operation {
    if raw.starts_with("tables.") {
        // GetFromTable
        let parts: Vec<&str> = raw.split(":").collect();
        if parts.len() != 2 {
            return Operation::Error("Invalid GetFromTable format".to_string());
        }

        let table_parts: Vec<&str> = parts[0].split(".").collect();
        if table_parts.len() < 3 {
            return Operation::Error("Invalid table specification".to_string());
        }

        let table = table_parts[1].to_string();
        let random_part = table_parts[2];

        let amount = random_part
            .trim_start_matches("random(")
            .trim_end_matches(")")
            .parse::<i32>()
            .unwrap_or(0);

        let filter = parts[1]
            .trim_start_matches("filter(")
            .trim_end_matches(")")
            .to_string();

        Operation::GetFromTable {
            table,
            filter,
            amount,
        }
    } else if raw.starts_with("game.players.") {
        if raw.contains(":state(") {
            // GetStateFromPlayer
            let parts: Vec<&str> = raw.split(":").collect();
            if parts.len() != 2 {
                return Operation::Error("Invalid GetStateFromPlayer format".to_string());
            }

            let selector = match parts[0] {
                s if s.ends_with(".current()") => Selector::Current,
                s if s.ends_with(".previous()") => Selector::Previous,
                s if s.ends_with(".next()") => Selector::Next,
                s if s.ends_with(".random()") => Selector::Random,
                _ => {
                    return Operation::Error("Invalid selector for GetStateFromPlayer".to_string())
                }
            };

            let state = parts[1]
                .trim_start_matches("state(")
                .trim_end_matches(")")
                .to_string();

            Operation::GetStateFromPlayer { selector, state }
        } else {
            // GetFromPlayers
            let selector = match raw.as_str() {
                s if s.ends_with(".current()") => Selector::Current,
                s if s.ends_with(".previous()") => Selector::Previous,
                s if s.ends_with(".next()") => Selector::Next,
                s if s.ends_with(".random()") => Selector::Random,
                _ => return Operation::Error("Invalid selector for GetFromPlayers".to_string()),
            };

            Operation::GetFromPlayers { selector }
        }
    } else if raw.starts_with("state.") {
        // UpdateState
        let parts: Vec<&str> = raw.split(":").collect();
        if parts.len() != 2 {
            return Operation::Error("Invalid UpdateState format".to_string());
        }

        let state_parts: Vec<&str> = parts[0].split(".").collect();
        if state_parts.len() != 3 {
            return Operation::Error("Invalid state specification".to_string());
        }

        let state = state_parts[1].to_string();
        let math_operation_str = state_parts[2]
            .trim_start_matches("update(")
            .trim_end_matches(")");

        let math_operation = match math_operation_str {
            "add" => MathOperation::Add,
            "sub" => MathOperation::Sub,
            "div" => MathOperation::Div,
            "mul" => MathOperation::Mul,
            _ => return Operation::Error("Invalid math operation".to_string()),
        };

        let value = parts[1]
            .trim_start_matches("value(")
            .trim_end_matches(")")
            .parse::<i32>()
            .unwrap_or(0);

        Operation::UpdateState {
            id: Uuid::new_v4(),
            state: state,
            math_operation: math_operation,
            value: value,
        }
    } else {
        Operation::Error("Unknown operation type".to_string())
    }
}

pub struct DeckBundle {
    pub tables: HashMap<String, Vec<Value>>,
    pub states: HashMap<String, StateModule>,
    pub cards: Vec<Vec<ParserSegment>>,
}

pub enum StateModule {
    LocalState {
        value: i64,
        min: i64,
        max: i64,
    },
    GlobalState {
        template: (i64, i64, i64),
        map: HashMap<Uuid, (i64, i64, i64)>,
    },
}

pub struct Value {
    pub value: String,
    tags: Vec<String>,
}

impl Value {
    pub fn new(value: String, tags: Vec<String>) {}
    pub fn has_tag(&self, tag: &String) -> bool {
        self.tags.contains(tag)
    }
}

#[derive(Parser)]
#[grammar = "grammar/segment.pest"]
struct DyncodeParser;
#[derive(Debug, PartialEq)]
pub enum ParserSegment {
    RawText(String),
    DynCode(String),
}

pub fn into_segments(raw_string: String) -> Vec<ParserSegment> {
    let mut segments = Vec::new();

    match DyncodeParser::parse(Rule::result, &raw_string) {
        Ok(parsed) => {
            // println!("Parsed successfully:\n{:#?}", parsed);
            for pair in parsed {
                print!(
                    "Rule: {:?}, Span: {:?}\n",
                    pair.as_rule(),
                    pair.as_span().as_str()
                );
                match pair.as_rule() {
                    Rule::result => {
                        for inner_pair in pair.into_inner() {
                            match inner_pair.as_rule() {
                                Rule::outside_braces => {
                                    let raw_text = inner_pair.as_str().to_string();
                                    segments.push(ParserSegment::RawText(raw_text));
                                }
                                //sigma code
                                Rule::braced_content => {
                                    let dyn_code = inner_pair
                                        .into_inner()
                                        .next()
                                        .unwrap()
                                        .as_str()
                                        .to_string();
                                    segments.push(ParserSegment::DynCode(dyn_code));
                                }
                                _ => {
                                    println!("Unknown rule: {:?}", inner_pair.as_rule());
                                }
                            }
                        }
                    }
                    _ => {
                        println!("Unknown rule: {:?}", pair.as_rule());
                    }
                }
            }
        }
        Err(e) => {
            println!("Error parsing: {}", e);
        }
    }

    segments
}
