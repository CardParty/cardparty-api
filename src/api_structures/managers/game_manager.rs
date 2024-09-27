use crate::api_structures::card_game::deck::MathOperation;
use crate::api_structures::card_game::deck::Operation;
use crate::api_structures::card_game::deck::Selector;
use crate::api_structures::card_game::deck::{
    into_operation, Card, DeckBundle, ParserSegment, StateModule, Value,
};
use crate::api_structures::session::Player;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateOption {
    pub id: Uuid,
    pub state: String,
    pub math_operation: MathOperation,
    pub value: i32,
}

pub struct CardResult {
    pub state_options: Vec<StateOption>,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    players: Vec<Player>,
    tables: HashMap<String, Vec<Value>>,
    states: HashMap<String, StateModule>,
    cards: Vec<Card>,
    card_count: usize,
    current: usize,
    score_state: String,
}

impl GameState {
    pub fn new(bundle: DeckBundle, cards: Vec<Card>) -> Self {
        Self {
            players: Vec::new(),
            tables: bundle.tables,
            states: bundle.states,
            cards,
            card_count: 0,
            current: 0,
            score_state: bundle.score_state,
        }
    }

    pub fn reset(&mut self) {
        self.card_count = 0;
        self.current = 0;
    }

    pub fn change_deck(&mut self, bundle: DeckBundle, cards: Vec<Card>) {
        self.tables = bundle.tables;
        self.states = bundle.states;
        self.score_state = bundle.score_state;
        self.cards = cards;
    }
}

#[derive(Clone)]
pub struct GameManager {
    rng: ThreadRng,
    game_state: GameState,
    awaited_states: HashMap<Uuid, StateOption>,
}

impl GameManager {
    pub fn init(bundle: DeckBundle) -> Self {
        let mut cards = Vec::new();
        for card in bundle.cards.clone() {
            let mut ops = Vec::new();
            for seg in card {
                match seg {
                    ParserSegment::DynCode(raw) => ops.push(into_operation(raw)),
                    ParserSegment::RawText(raw) => {
                        ops.push(Operation::RawText(
                            raw,
                        ));
                    }
                };
            }
            cards.push(Card {
                operations: ops,
                id: Uuid::new_v4(),
            });
        }

        Self {
            rng: thread_rng(),
            game_state: GameState::new(bundle.clone(), cards),
            awaited_states: HashMap::new(),
        }
    }
    // pub fn generate_position_state(&self) -> HashMap<Uuid, i32> {
    //     let player_id = self.match_player(Selector::Current).id;
    //     if let Some(state) = self.awaited_states.get(&player_id) {
    //         let new_value = state.value as i64;
    //         if let Some(state_module) = self.game_state.states.get_mut(&state.state) {
    //             match state_module {
    //                 StateModule::LocalState {
    //                     value,
    //                     min: _,
    //                     max: _,
    //                 } => {
    //                     *value = match state.get() {
    //                         MathOperation::Add => *value + new_value,
    //                         MathOperation::Sub => *value - new_value,
    //                         MathOperation::Div => *value / new_value,
    //                         MathOperation::Mul => *value * new_value,
    //                     };
    //                 }
    //                 StateModule::GlobalState { template: _, map } => {
    //                     let (value, min, max) = map.get_mut(&player_id).unwrap();
    //                     *value = match state.math_operation {
    //                         MathOperation::Add => *value + new_value,
    //                         MathOperation::Sub => *value - new_value,
    //                         MathOperation::Div => *value / new_value,
    //                         MathOperation::Mul => *value * new_value,
    //                     };
    //                 }
    //             }
    //         }
    //     }
    //     self.awaited_states.clear();
    // }
    pub fn next_player(&mut self) {
        self.game_state.current += 1;
        if self.game_state.current >= self.game_state.players.len() {
            self.game_state.current = 0;
        }
    }

    pub fn resolve_state(&mut self, id: Uuid) {
        let player_id = self.match_player(Selector::Current).id;
        if let Some(state) = self.awaited_states.get(&id) {
            let new_value = state.value as i64;
            if let Some(state_module) = self.game_state.states.get_mut(&state.state) {
                match state_module {
                    StateModule::LocalState {
                        value,
                        min: _,
                        max: _,
                    } => {
                        *value = match state.math_operation {
                            MathOperation::Add => *value + new_value,
                            MathOperation::Sub => *value - new_value,
                            MathOperation::Div => *value / new_value,
                            MathOperation::Mul => *value * new_value,
                        };
                    }
                    StateModule::GlobalState { template: _, map } => {
                        let (value, _min, _max) = map.get_mut(&player_id).unwrap();
                        *value = match state.math_operation {
                            MathOperation::Add => *value + new_value,
                            MathOperation::Sub => *value - new_value,
                            MathOperation::Div => *value / new_value,
                            MathOperation::Mul => *value * new_value,
                        };
                    }
                }
            }
        }
        self.awaited_states.clear();
    }

    pub fn start_game(&mut self) {
        self.game_state.players.shuffle(&mut self.rng);
        self.game_state.cards.shuffle(&mut self.rng);
    }

    pub fn add_player(&mut self, id: Uuid, username: String, is_host: bool) {
        self.game_state
            .players
            .push(Player::new(id, username, is_host));

        for (_, v) in self.game_state.states.iter_mut() {
            if let StateModule::GlobalState { template, map } = v {
                map.insert(id, template.clone());
            }
        }
    }

    pub fn remove_player(&mut self, id: Uuid) {
        self.game_state.players.retain(|p| p.id != id);
        for (_, v) in self.game_state.states.iter_mut() {
            if let StateModule::GlobalState { template: _, map } = v {
                map.retain(|k, _| *k != id);
            }
        }
    }

    fn match_player(&self, selector: Selector) -> Player {
        match selector {
            Selector::Current => self.game_state.players[self.game_state.current].clone(),
            Selector::Next => {
                let next = (self.game_state.current + 1) % self.game_state.players.len();
                self.game_state.players[next].clone()
            }
            Selector::Previous => {
                let prev = if self.game_state.current == 0 {
                    self.game_state.players.len() - 1
                } else {
                    self.game_state.current - 1
                };
                self.game_state.players[prev].clone()
            }
            Selector::Random => {
                let mut rng = thread_rng();
                self.game_state.players.choose(&mut rng).unwrap().clone()
            }
        }
    }

    pub fn get_next_card(&mut self) -> CardResult {
        let card =
            self.game_state.cards[self.game_state.card_count % self.game_state.cards.len()].clone();
        self.game_state.card_count += 1;

        let mut buffer: Vec<String> = Vec::new();
        let mut state_options: Vec<StateOption> = Vec::new();

        for op in card.operations {
            match op {
                Operation::GetFromTable {
                    table,
                    filter,
                    amount,
                } => {
                    let selection: usize = if amount == 0 { 1 } else { amount as usize };
                    let table = self.game_state.tables.get(&table).unwrap();
                    let values: Vec<String> = table
                        .iter()
                        .filter(move |val| filter.is_empty() || val.has_tag(&filter))
                        .collect::<Vec<&Value>>()
                        .choose_multiple(&mut self.rng, selection)
                        .cloned()
                        .map(|value| value.value.clone())
                        .collect();

                    buffer.push(values.join(", "));
                }
                Operation::GetFromPlayers { selector } => {
                    buffer.push(self.match_player(selector).username);
                }
                Operation::GetStateFromPlayer { selector, state } => {
                    let player = self.match_player(selector);
                    match self.game_state.states.get(&state).expect("State not found") {
                        StateModule::LocalState {
                            value,
                            min: _,
                            max: _,
                        } => {
                            buffer.push(format!("{}: {}", state, value));
                        }
                        StateModule::GlobalState { template: _, map } => {
                            if let Some(value) = map.get(&player.id) {
                                buffer.push(format!("{}: {} {}", player.username, value.0, state));
                            }
                        }
                    }
                }
                Operation::UpdateState {
                    id,
                    state,
                    math_operation,
                    value,
                } => {
                    self.awaited_states.insert(
                        id,
                        StateOption {
                            id,
                            state,
                            math_operation,
                            value,
                        },
                    );
                }
                Operation::Error(err) => {
                    buffer.clear();
                    buffer.push(err);
                }
                Operation::RawText(raw) => buffer.push(raw),
            }
        }
        CardResult {
            state_options,
            text: buffer.join(""),
        }
    }

    pub fn change_deck(&mut self, bundle: DeckBundle) {
        let mut cards = Vec::new();
        for card in bundle.cards.clone() {
            let mut ops = Vec::new();
            for seg in card {
                match seg {
                    ParserSegment::DynCode(raw) => ops.push(into_operation(raw)),
                    ParserSegment::RawText(raw) => {
                        ops.push(Operation::RawText(
                            raw,
                        ));
                    }
                };
            }
            cards.push(Card {
                operations: ops,
                id: Uuid::new_v4(),
            });
        }

        self.game_state.change_deck(bundle, cards);
    }

    pub fn reset_game_state(&mut self) {
        self.game_state.reset();
    }
}
