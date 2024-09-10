use serde::de::value;
use uuid::Uuid;
use std::collections::HashMap;

use crate::api_structures::{card_game::deck::Deck, id::UserId};
pub struct InitalState {
    value: String,
}

impl InitalState {
    pub fn new(value: String) -> Self {
        Self { value: value }
    }

    pub fn into_state(self) -> State {
        State::new(self.value)
    }
}
pub struct State {
    value: String,
    id: Uuid,
}

impl State {
    pub fn new(value: String) -> (Self, Uuid) {
        let id = Uuid::new_v4();
        (
            Self {
                value: value,
                id: id,
            },
            id,
        )
    }
}

pub struct Tag {
    name: String,
}

pub struct Value {
    value: String,
    tags: Vec<Tag>,
}

pub struct Table {
    values: Vec<Value>,
}
pub struct CachedTable {
  tag_caches: HashMap<String, Vec<Value>>
}

impl CachedTable {
  pub fn generate_cache(table: Table) -> Self {
      let mut tag_map: HashMap<String, Vec<Value>> = HashMap::new();

      for value in table.values.iter() {
          for tag in value.tags.iter() {
              tag_map.entry(tag.name.clone())
                  .or_insert_with(Vec::new)
                  .push(value.clone());
          }
      }
      
      CachedTable { tag_caches: tag_map }
  }

  pub fn query(&self, tag: String) -> Option<&Vec<Value>> {
    if let Some(table) = self.tag_caches.get(&tag) {
      Some(table)
    } else {
      None
    }
  }
  
}

pub struct Card {
  metaCode: MetaCode
}

pub struct Deck {
    cards: Vec<Card>,
    inital_state: Vec<InitalState>,
    tables: Vec<CachedTable>,
}

pub struct GameManager {
    deck: Deck,
    state: Vec<State>,
    current_player: usize,
}

impl GameManager {
    pub fn update_deck(&mut self, deck: Deck) {
        self.state = Vec::new();
        for inital_state in deck.inital_state {
            self.state.push(inital_state.into_state());
        }
        self.deck = deck;
    }

    pub fn get_state(&self) -> Vec<State> {
        self.state.clone()
    }

    pub fn update_state(&mut self, state_id: Uuid, value: String) {
        for state in self.state.iter_mut() {
            if state.id == state_id {
                state.value = value;
            }
            break;
        }
    }

    pub fn 
}

pub struct GameManagerBuilder {
    deck: Option<Deck>,
    state: Vec<State>,
}

impl GameManagerBuilder {
    pub fn new() -> Self {
        Self {
            deck: None,
            state: Vec::new(),
        }
    }

    pub fn state(mut self, inital_state: InitalState) -> Self {
        self.state.push(inital_state.into_state());
        self
    }

    pub fn deck(mut self, deck: Deck) -> Self {
        self.deck = Some(deck);
        for inital_state in deck.inital_state.drain(..) {
            self.state(inital_state)
        }
        self
    }

    pub fn build(self) -> GameManager {
        if let Some(deck) = self.deck {
            GameManager {
                current_player: 0,
                state,
                deck,
            }
        } else {
            panic!("im bouta kms")
        }
    }
}
