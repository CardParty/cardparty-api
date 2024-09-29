use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api_structures::session::Player;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Data {
    String { string: String },
    Integer { integer: i32 },
    StateRefrence { ident: String },
    TableRefrence { ident: String },
    ActionRefrence { ident: String },
    Combinator { buff: Vec<Data> },
}

impl Data {
    pub fn get_ident(&self) -> Option<String> {
        match self {
            Data::String { string } => Some(string.clone()),
            Data::StateRefrence { ident } => Some(ident.clone()),
            Data::TableRefrence { ident } => Some(ident.clone()),
            Data::ActionRefrence { ident } => Some(ident.clone()),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "segment")]
pub enum Segment {
    Raw { string: String },
    Action { ident: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "selector")]
pub enum Selector {
    Current,
    Previous,
    Next,
    Random,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Action {
    UpdateState {
        ident: String,
        state: String,
        value: Data,
        add: bool,
        selector: Selector,
    },
    Option {
        ident: String,
        display: String,
        actions: Vec<String>,
    },
    GetFromTable {
        ident: String,
        table: String,
        tags: Vec<String>,
    },
    GetFromState {
        ident: String,
        state: String,
        selector: Selector,
    },
}

impl Action {
    pub fn get_ident(&self) -> String {
        match self {
            Action::UpdateState { ident, .. } => ident.clone(),
            Action::Option { ident, .. } => ident.clone(),
            Action::GetFromTable { ident, .. } => ident.clone(),
            Action::GetFromState { ident, .. } => ident.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub value: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Table {
    pub ident: String,
    pub values: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub segments: Vec<Segment>,
    pub actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub ident: String,
    pub value: Data,
    pub individual: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    pub deck_name: String,
    pub id: Uuid,
    pub scoreboard: ScoreBoard,
    max_cards: i32,
    max_players: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ScoreBoardCondition {
    Biggest,
    Lowest,
    Closest,
    None,
    FirstToReach,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScoreElement {
    username: String,
    value: i32,
    position: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderedScoreBoard {
    ident: String,
    data: Vec<ScoreElement>,
}

impl Default for RenderedScoreBoard {
    fn default() -> Self {
        Self {
            ident: "".to_string(),
            data: Vec::new(),
        }
    }
}

impl PartialEq for ScoreElement {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for ScoreElement {}

impl PartialOrd for ScoreElement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl Ord for ScoreElement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScoreBoard {
    state: Data,
    value: Data, // this is the value that will be used for the condition, like ClosestTo will use this value to compare. In conditions like FirstToReach this value will be the target value, in conditions like Biggest this will be ignored as it will just find the biggest value
    cond: ScoreBoardCondition,
}

impl ScoreBoard {
    pub fn generate_scoreboard(
        &self,
        states: HashMap<String, StateModule>,
        players: Vec<Player>,
    ) -> Result<RenderedScoreBoard, ()> {
        println!("{:#?}", self);
        let score_state = states.get(&self.state.get_ident().ok_or(())?).ok_or(())?;
        let mut score_elements: Vec<ScoreElement> = Vec::new();
        match score_state {
            StateModule::IndividualState {
                constructor_value,
                map,
            } => {

                let mut buffer: Vec<(Uuid, i32)> = Vec::new();
                for player in &players {
                    let plr_value = map.get(&player.id).unwrap_or(constructor_value);
                    buffer.push((player.id, *plr_value));

                }
                buffer.sort_by(|a, b| a.1.cmp(&b.1));

                match self.cond {
                    ScoreBoardCondition::Biggest => {
                        let cloned_buffer = buffer.clone();
                        cloned_buffer.iter().enumerate().for_each(|(i, (id, value))| {
                            score_elements.push(ScoreElement {
                                username: players
                                    .iter()
                                    .find(|x| x.id == *id)
                                    .unwrap()
                                    .username
                                    .clone(),
                                value: *value,
                                position: i as i32,
                            });
                        });
                    },
                    ScoreBoardCondition::Lowest => {
                        let cloned_buffer = buffer.clone();
                        buffer.reverse();
                        cloned_buffer.iter().enumerate().for_each(|(i, (id, value))| {
                            score_elements.push(ScoreElement {
                                username: players
                                  .iter()
                                  .find(|x| x.id == *id)
                                  .unwrap()
                                  .username
                                  .clone(),
                                value: *value,
                                position: i as i32,
                            });
                        });
                    }
                    _ => { return Err(()) }
                }
            }
            _ => { }
        }

        Ok(RenderedScoreBoard {
            ident: self.state.get_ident().ok_or(())?,
            data: score_elements,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeckBundle {
    pub score_state: ScoreBoard,
    pub tables: HashMap<String, Vec<Value>>,
    pub states: HashMap<String, StateModule>,
    pub cards: Vec<Card>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
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
            table_hash.insert(table.ident, table.values);
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
            score_state: self.meta.scoreboard,
            tables: table_hash,
            states: state_hash,
            cards: self.cards,
        }
    }
}
