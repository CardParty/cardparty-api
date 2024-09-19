use std::collections::HashMap;

use crate::api_structures::card_game::deck::{DeckBundle, StateModule};
use crate::api_structures::session::Player;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
pub struct GameManager {
    players: Vec<Player>,
    tables: HashMap<String, Vec<Value>>,
    states: HashMap<String, StateModule>,
    cards: Vec<Card>,
    current: usize,
    rng: ThreadRng,
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
}
