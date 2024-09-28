// use pest::Parser;
// use pest_derive::Parser;
// use serde::{Deserialize, Serialize};
// use std::{collections::HashMap, hash::Hash};
// use uuid::Uuid;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum Selector {
//     Current,
//     Previous,
//     Next,
//     Random,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum MathOperation {
//     Add,
//     Sub,
//     Div,
//     Mul,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum Operation {
//     GetFromTable {
//         table: String,
//         filter: String,
//         amount: i32,
//     },
//     GetFromPlayers {
//         selector: Selector,
//     },
//     GetStateFromPlayer {
//         selector: Selector,
//         state: String,
//     },
//     UpdateState {
//         id: Uuid,
//         state: String,
//         math_operation: MathOperation,
//         value: i32,
//     },
//     Error(String),
//     RawText(String),
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Card {
//     pub operations: Vec<Operation>,
//     pub id: Uuid,
// }

// pub fn into_operation(raw: String) -> Operation {
//     if raw.starts_with("tables.") {
//         // GetFromTable
//         let parts: Vec<&str> = raw.split(":").collect();
//         if parts.len() != 2 {
//             return Operation::Error("Invalid GetFromTable format".to_string());
//         }

//         let table_parts: Vec<&str> = parts[0].split(".").collect();
//         if table_parts.len() < 3 {
//             return Operation::Error("Invalid table specification".to_string());
//         }

//         let table = table_parts[1].to_string();
//         let random_part = table_parts[2];

//         let amount = random_part
//             .trim_start_matches("random(")
//             .trim_end_matches(")")
//             .parse::<i32>()
//             .unwrap_or(0);

//         let filter = parts[1]
//             .trim_start_matches("filter(")
//             .trim_end_matches(")")
//             .to_string();

//         Operation::GetFromTable {
//             table,
//             filter,
//             amount,
//         }
//     } else if raw.starts_with("game.players.") {
//         if raw.contains(":state(") {
//             // GetStateFromPlayer
//             let parts: Vec<&str> = raw.split(":").collect();
//             if parts.len() != 2 {
//                 return Operation::Error("Invalid GetStateFromPlayer format".to_string());
//             }

//             let selector = match parts[0] {
//                 s if s.ends_with(".current()") => Selector::Current,
//                 s if s.ends_with(".previous()") => Selector::Previous,
//                 s if s.ends_with(".next()") => Selector::Next,
//                 s if s.ends_with(".random()") => Selector::Random,
//                 _ => {
//                     return Operation::Error("Invalid selector for GetStateFromPlayer".to_string())
//                 }
//             };

//             let state = parts[1]
//                 .trim_start_matches("state(")
//                 .trim_end_matches(")")
//                 .to_string();

//             Operation::GetStateFromPlayer { selector, state }
//         } else {
//             // GetFromPlayers
//             let selector = match raw.as_str() {
//                 s if s.ends_with(".current()") => Selector::Current,
//                 s if s.ends_with(".previous()") => Selector::Previous,
//                 s if s.ends_with(".next()") => Selector::Next,
//                 s if s.ends_with(".random()") => Selector::Random,
//                 _ => return Operation::Error("Invalid selector for GetFromPlayers".to_string()),
//             };

//             Operation::GetFromPlayers { selector }
//         }
//     } else if raw.starts_with("state.") {
//         // UpdateState
//         let parts: Vec<&str> = raw.split(":").collect();
//         if parts.len() != 2 {
//             return Operation::Error("Invalid UpdateState format".to_string());
//         }

//         let state_parts: Vec<&str> = parts[0].split(".").collect();
//         if state_parts.len() != 3 {
//             return Operation::Error("Invalid state specification".to_string());
//         }

//         let state = state_parts[1].to_string();
//         let math_operation_str = state_parts[2]
//             .trim_start_matches("update(")
//             .trim_end_matches(")");

//         let math_operation = match math_operation_str {
//             "add" => MathOperation::Add,
//             "sub" => MathOperation::Sub,
//             "div" => MathOperation::Div,
//             "mul" => MathOperation::Mul,
//             _ => return Operation::Error("Invalid math operation".to_string()),
//         };

//         let value = parts[1]
//             .trim_start_matches("value(")
//             .trim_end_matches(")")
//             .parse::<i32>()
//             .unwrap_or(0);

//         Operation::UpdateState {
//             id: Uuid::new_v4(),
//             state,
//             math_operation,
//             value,
//         }
//     } else {
//         Operation::Error("Unknown operation type".to_string())
//     }
// }

// #[derive(Debug, Clone)]

// pub struct DeckBundle {
//     pub score_state: String,
//     pub tables: HashMap<String, Vec<Value>>,
//     pub states: HashMap<String, StateModule>,
//     pub cards: Vec<Vec<ParserSegment>>,
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum StateModule {
//     LocalState {
//         value: i64,
//         min: i64,
//         max: i64,
//     },
//     GlobalState {
//         template: (i64, i64, i64),
//         map: HashMap<Uuid, (i64, i64, i64)>,
//     },
// }
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Value {
//     pub value: String,
//     tags: Vec<String>,
// }

// impl Value {
//     pub fn new(value: String, tags: Vec<String>) {}
//     pub fn has_tag(&self, tag: &String) -> bool {
//         self.tags.contains(tag)
//     }
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Deck {
//     deck_name: String,
//     id: Uuid,
//     score_state: String,
//     tables: Vec<Table>,
//     states: Vec<State>,
//     cards: Vec<String>,
// }

// impl Deck {
//     pub fn into_bundle(self) -> DeckBundle {
//         let mut table_hash = HashMap::new();
//         for table in self.tables {
//             table_hash.insert(table.name, table.values);
//         }
//         let mut state_hash = HashMap::new();
//         for state in self.states {
//             if state.is_local {
//                 state_hash.insert(
//                     state.name,
//                     StateModule::LocalState {
//                         value: state.value as i64,
//                         min: 0,
//                         max: 0,
//                     },
//                 );
//             }
//         }

//         let mut segmentized_cards = Vec::new();
//         for card in self.cards {
//             segmentized_cards.push(into_segments(card));
//         }

//         DeckBundle {
//             score_state: self.score_state,
//             tables: table_hash,
//             states: state_hash,
//             cards: segmentized_cards,
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct State {
//     pub name: String,
//     pub value: i32,
//     pub is_local: bool,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Table {
//     pub name: String,
//     pub values: Vec<Value>,
// }

// #[derive(Parser)]
// #[grammar = "grammar/segment.pest"]
// struct DyncodeParser;
// #[derive(Debug, PartialEq, Clone)]
// pub enum ParserSegment {
//     RawText(String),
//     DynCode(String),
// }

// pub fn into_segments(raw_string: String) -> Vec<ParserSegment> {
//     let mut segments = Vec::new();

//     match DyncodeParser::parse(Rule::result, &raw_string) {
//         Ok(parsed) => {
//             // println!("Parsed successfully:\n{:#?}", parsed);
//             for pair in parsed {
//                 print!(
//                     "Rule: {:?}, Span: {:?}\n",
//                     pair.as_rule(),
//                     pair.as_span().as_str()
//                 );
//                 match pair.as_rule() {
//                     Rule::result => {
//                         for inner_pair in pair.into_inner() {
//                             match inner_pair.as_rule() {
//                                 Rule::outside_braces => {
//                                     let raw_text = inner_pair.as_str().to_string();
//                                     segments.push(ParserSegment::RawText(raw_text));
//                                 }
//                                 //sigma code
//                                 Rule::braced_content => {
//                                     let dyn_code = inner_pair
//                                         .into_inner()
//                                         .next()
//                                         .unwrap()
//                                         .as_str()
//                                         .to_string();
//                                     segments.push(ParserSegment::DynCode(dyn_code));
//                                 }
//                                 _ => {
//                                     println!("Unknown rule: {:?}", inner_pair.as_rule());
//                                 }
//                             }
//                         }
//                     }
//                     _ => {
//                         println!("Unknown rule: {:?}", pair.as_rule());
//                     }
//                 }
//             }
//         }
//         Err(e) => {
//             println!("Error parsing: {}", e);
//         }
//     }

//     segments
// }

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Data {
    String { string: String },
    Integer { integer: i32 },
    StateRefrence { ident: String },
    TableRefrence { ident: String },
    ActionRefrence { ident: String },
    Combinator { buff: Vec<Data> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "segment")]
pub enum Segment {
    Raw { string: String },
    Action { ident: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "selector")]
pub enum Selector {
    Current,
    Previous,
    Next,
    Random,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Action {
    UpdateState {
        state: Data,
        add: bool,
        selector: Selector,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Value {
    value: String,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub name: String,
    pub values: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    segments: Vec<Segment>,
    actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub ident: String,
    pub value: Data,
    pub individual: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    pub deck_name: String,
    pub id: Uuid,
    pub score_board: ScoreBoard,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ScoreBoardCondition {
    Biggest,
    Lowest,
    Closest,
    None,
    FirstToReach,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScoreBoard {
    state: Data,
    value: Data,
    cond: ScoreBoardCondition,
}

pub enum StateModule {
    SharedState {
        ident: String,
        value: i32,
    },
    IndividualState {
        constructor_value: i32,
        map: HashMap<Uuid, i32>,
    },
}
pub struct DeckBundle {
    pub score_state: ScoreBoard,
    pub tables: HashMap<String, Vec<Value>>,
    pub states: HashMap<String, StateModule>,
    pub cards: Vec<Card>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Deck {
    meta: Meta,
    tables: Vec<Table>,
    states: Vec<State>,
    cards: Vec<Card>,
}

impl Deck {
    pub fn into_bundle(self) -> DeckBundle {
        let mut table_hash = HashMap::new();
        for table in self.tables {
            table_hash.insert(table.name, table.values);
        }
        let mut state_hash = HashMap::new();
        for state in self.states {
            if state.individual {
                state_hash.insert(
                    state.ident,
                    StateModule::IndividualState {
                        constructor_value: match state.value {
                            Data::Integer { integer } => integer,
                            _ => 0,
                        },
                        map: HashMap::new(),
                    },
                );
            } else {
                state_hash.insert(
                    state.ident.clone(),
                    StateModule::SharedState {
                        ident: state.ident.clone(),
                        value: match state.value {
                            Data::Integer { integer } => integer,
                            _ => 0,
                        },
                    },
                );
            }
        }

        DeckBundle {
            score_state: self.meta.score_board,
            tables: table_hash,
            states: state_hash,
            cards: self.cards,
        }
    }
}
