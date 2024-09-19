use crate::api_structures::card_game::deck::MathOperation;
use crate::api_structures::card_game::deck::Operation;
use crate::api_structures::card_game::deck::Selector;
use crate::api_structures::card_game::deck::{
    into_operation, Card, DeckBundle, ParserSegment, StateModule, Value,
};
use crate::api_structures::session::Player;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::hash::Hash;
use uuid::Uuid;
#[derive(Debug, Clone)]
pub enum GameState {
    Lobby,
    PreGame,
    Game,
    PostGame,
}
#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub struct GameManager {
    players: Vec<Player>,
    tables: HashMap<String, Vec<Value>>,
    states: HashMap<String, StateModule>,
    cards: Vec<Card>,
    card_count: usize,
    current: usize,
    rng: ThreadRng,
    game_state: GameState,
    awaited_states: HashMap<Uuid, StateOption>,
}

impl GameManager {
    pub fn init(bundle: DeckBundle) -> Self {
        let mut cards = Vec::new();
        for card in bundle.cards {
            let mut ops = Vec::new();
            for seg in card {
                match seg {
                    ParserSegment::DynCode(raw) => ops.push(into_operation(raw)),
                    ParserSegment::RawText(raw) => {
                        ops.push(crate::api_structures::card_game::deck::Operation::RawText(
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
            players: Vec::new(),
            tables: bundle.tables,
            states: bundle.states,
            cards: cards,
            card_count: 0,
            current: 0,
            rng: thread_rng(),
            game_state: GameState::Lobby,
            awaited_states: HashMap::new(),
        }
    }

    pub fn next_player(&mut self) {
        self.current += 1;
        if self.current < self.players.len() {
            self.current = 0;
        }
    }

    pub fn resolve_state(&mut self, id: Uuid) {
        let player_id = self.match_player(Selector::Current).id;
        if let Some(state) = self.awaited_states.get(&id) {
            let new_value = state.value as i64;
            if let Some(state_module) = self.states.get_mut(&state.state) {
                match state_module {
                    StateModule::LocalState { value, min, max } => {
                        match state.math_operation {
                            crate::api_structures::card_game::deck::MathOperation::Add => {
                                *value + new_value
                            }
                            crate::api_structures::card_game::deck::MathOperation::Sub => {
                                *value - new_value
                            }
                            crate::api_structures::card_game::deck::MathOperation::Div => {
                                *value / new_value
                            }
                            crate::api_structures::card_game::deck::MathOperation::Mul => {
                                *value * new_value
                            }
                        };
                    }
                    StateModule::GlobalState { template, map } => {
                        let (value, min, max) = map.get_mut(&player_id).unwrap();
                        match state.math_operation {
                            crate::api_structures::card_game::deck::MathOperation::Add => {
                                *value + new_value
                            }
                            crate::api_structures::card_game::deck::MathOperation::Sub => {
                                *value - new_value
                            }
                            crate::api_structures::card_game::deck::MathOperation::Div => {
                                *value / new_value
                            }
                            crate::api_structures::card_game::deck::MathOperation::Mul => {
                                *value * new_value
                            }
                        };
                    }
                }
            }
        }
        self.awaited_states = HashMap::new();
    }

    pub fn start_game(&mut self) {
        self.players.shuffle(&mut self.rng);
        self.cards.shuffle(&mut self.rng);
    }

    pub fn add_player(&mut self, id: Uuid, username: String, is_host: bool) {
        self.players.push(Player::new(id, username, is_host));

        for (k, v) in self.states.iter_mut() {
            match v {
                StateModule::GlobalState { template, map } => {
                    map.insert(id, template.clone());
                }
                _ => {}
            };
        }
    }

    pub fn remove_player(&mut self, id: Uuid) {
        self.players.retain(|p| p.id != id);
        for (k, v) in self.states.iter_mut() {
            match v {
                StateModule::GlobalState { template, map } => {
                    map.retain(|k, v| *k != id);
                }
                _ => {}
            };
        }
    }

    pub fn advance_state(&mut self) {
        match self.game_state {
            GameState::Lobby => self.game_state = GameState::PreGame,
            GameState::PreGame => self.game_state = GameState::Game,
            GameState::Game => self.game_state = GameState::PostGame,
            GameState::PostGame => self.game_state = GameState::Lobby,
        }
    }

    pub fn get_state(self) -> GameState {
        self.game_state
    }

    fn match_player(&mut self, selector: Selector) -> Player {
        match selector {
            Selector::Current => return self.players[self.current].clone(),
            Selector::Next => {
                if self.current == self.players.len() {
                    return self.players[0].clone();
                } else {
                    return self.players[self.current + 1].clone();
                }
            }
            Selector::Previous => {
                if self.current == 0 {
                    return self.players[self.players.len() - 1].clone();
                } else {
                    return self.players[self.current - 1].clone();
                }
            }
            Selector::Random => {
                let mut players = self.players.clone();
                players.shuffle(&mut self.rng);
                return players[0].clone();
            }
        }
    }

    pub fn get_next_card(mut self) -> CardResult {
        let card = self.cards[self.card_count % self.cards.len()].clone();

        let mut buffer: Vec<String> = Vec::new();
        let mut state_options: Vec<StateOption> = Vec::new();

        for op in card.operations {
            match op {
                Operation::GetFromTable {
                    table,
                    filter,
                    amount,
                } => {
                    let selection: usize = if amount == 0 {
                        (amount + 1) as usize
                    } else {
                        amount as usize
                    };
                    let table = self.tables.get(&table).unwrap();
                    let values: Vec<String> = table
                        .iter()
                        .filter(move |val| {
                            if filter.is_empty() {
                                true
                            } else {
                                val.has_tag(&filter)
                            }
                        })
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
                    match self.states.get(&state).expect("bomba wybuchło") {
                        StateModule::LocalState { value, min, max } => {
                            buffer.push(format!("{}: {}", state, value));
                        }
                        StateModule::GlobalState { template, map } => {
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
            state_options: state_options,
            text: buffer.join(""),
        }
    }
}
