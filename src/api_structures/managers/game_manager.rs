use std::collections::HashMap;

use crate::api_structures::card_game::deck::{Card, DeckBundle, StateModule, Value};
use crate::api_structures::session::Player;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use uuid::Uuid;

pub enum GameState {
    AwaitHost,
    AwaitDeck,
}

pub struct GameManager {
    players: Vec<Player>,
    tables: HashMap<String, Vec<Value>>,
    states: HashMap<String, StateModule>,
    cards: Vec<Card>,
    current: usize,
    rng: ThreadRng,
    game_state: GameState,
}

impl GameManager {
    pub fn init(bundle: DeckBundle) -> Self {
        Self {
            players: Vec::new(),
            tables: (),
            states: (),
            cards: (),
            current: 0,
            rng: thread_rng(),
        }
    }

    pub fn next_player(&mut self) {
        self.current += 1;
        if self.current < self.players.len() {
            self.current = 0;
        }
    }

    pub fn start_game(&mut self) {
        self.players.shuffle(&mut self.rng);
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
}
