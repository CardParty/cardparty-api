use crate::api_structures::card_game::deck::{
    Action, Card, Data, DeckBundle, Segment, StateModule, Value,
};
use crate::api_structures::card_game::deck::{ScoreBoard, Selector};
use crate::api_structures::id;
use crate::api_structures::session::Player;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intermediate {
    Value(String),
    Action(Action),
}

impl Intermediate {
    pub fn to_string(&self) -> Option<String> {
        match self {
            Intermediate::Value(value) => Some(value.clone()),
            Intermediate::Action(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub ident: String,
    pub modifier: i32,
    pub selector: Selector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardOption {
    pub id: Uuid,
    pub display: String,
    #[serde(skip)]
    pub updates: Vec<StateUpdate>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardResult {
    pub state_options: Vec<CardOption>,
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
    score_state: ScoreBoard,
}

impl GameState {
    pub fn new(bundle: DeckBundle) -> Self {
        Self {
            players: Vec::new(),
            tables: bundle.tables,
            states: bundle.states,
            cards: bundle.cards,
            card_count: 0,
            current: 0,
            score_state: bundle.score_state,
        }
    }

    pub fn reset(&mut self) {
        self.card_count = 0;
        self.current = 0;
    }

    pub fn change_deck(&mut self, bundle: DeckBundle) {
        self.tables = bundle.tables;
        self.states = bundle.states;
        self.score_state = bundle.score_state;
        self.cards = bundle.cards;
    }
}

#[derive(Clone)]
pub struct GameManager {
    rng: ThreadRng,
    game_state: GameState,
    awaited_states: HashMap<Uuid, CardOption>,
}

impl GameManager {
    pub fn init(bundle: DeckBundle) -> Self {
        Self {
            rng: thread_rng(),
            game_state: GameState::new(bundle.clone()),
            awaited_states: HashMap::new(),
        }
    }
    pub fn next_player(&mut self) {
        self.game_state.current += 1;
        if self.game_state.current >= self.game_state.players.len() {
            self.game_state.current = 0;
        }
    }

    pub fn resolve_state(&mut self, id: Uuid) {
        if let Some(option) = self.awaited_states.remove(&id) {
            for update in option.updates {
                match update.selector {
                    Selector::None => {
                        let state = self.game_state.states.get_mut(&update.ident).unwrap();
                        match state {
                            StateModule::SharedState { value, ident } => {
                                *value += update.modifier;
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        let player = self.match_player(update.selector);
                        let state = self.game_state.states.get_mut(&update.ident).unwrap();
                        match state {
                            StateModule::IndividualState {
                                constructor_value,
                                map,
                            } => {
                                if let Some(value) = map.get_mut(&player.id) {
                                    *value += update.modifier;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
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
            match v {
                StateModule::IndividualState {
                    constructor_value,
                    map,
                } => {
                    map.insert(id, constructor_value.clone());
                }
                _ => {}
            }
        }
    }

    pub fn remove_player(&mut self, id: Uuid) {
        self.game_state.players.retain(|p| p.id != id);
        for (_, v) in self.game_state.states.iter_mut() {
            match v {
                StateModule::IndividualState {
                    constructor_value: _,
                    map,
                } => {
                    map.retain(|k, _| *k != id);
                }
                _ => {}
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
            Selector::None => self.game_state.players[self.game_state.current].clone(),
        }
    }

    pub fn get_next_card(&mut self) -> Option<CardResult> {
        let mut buffer: Vec<String> = Vec::new();
        let mut decisions: Vec<CardOption> = Vec::new();

        let mut actions_cache: HashMap<String, Intermediate> = HashMap::new();
        let mut state_updates: HashMap<String, StateUpdate> = HashMap::new();

        if let Some(card) = self.game_state.cards.choose(&mut self.rng) {
            for action in card.actions.clone() {
                if let Some((inter, ident)) = match action {
                    Action::GetFromTable { ident, table, tags } => {
                        let filtered_table = self.game_state.tables.get(&table).map(|v| {
                            if tags.is_empty() {
                                v.clone()
                            } else {
                                v.iter()
                                    .filter(|item| tags.iter().all(|tag| item.tags.contains(tag)))
                                    .cloned()
                                    .collect()
                            }
                        });

                        if let Some(table) = filtered_table {
                            let value = table.choose(&mut self.rng).unwrap().value.clone();
                            Some((Intermediate::Value(value), ident))
                        } else {
                            Some((
                                Intermediate::Value("WYWYWYWYWYYW WYJEBAŁO SIE".to_string()),
                                ident,
                            ))
                        }
                    }
                    Action::GetFromState {
                        ident,
                        state,
                        selector,
                    } => match selector {
                        Selector::None => {
                            let state = self.game_state.states.get(&state).unwrap();
                            match state {
                                StateModule::SharedState {
                                    value,
                                    ident: state_ident,
                                } => Some((
                                    Intermediate::Value(value.clone().to_string()),
                                    state_ident.clone(),
                                )),
                                _ => Some((
                                    Intermediate::Value("WYWYWYWYWYYW WYJEBAŁO SIE v3".to_string()),
                                    ident,
                                )),
                            }
                        }
                        _ => {
                            let player = self.match_player(selector);
                            let state = self.game_state.states.get(&state).unwrap();
                            match state {
                                StateModule::IndividualState {
                                    constructor_value,
                                    map,
                                } => {
                                    if let Some(value) = map.get(&player.id) {
                                        Some((
                                            Intermediate::Value(value.clone().to_string()),
                                            ident,
                                        ))
                                    } else {
                                        Some((
                                            Intermediate::Value(
                                                "WYWYWYWYWYYW WYJEBAŁO SIE V2".to_string(),
                                            ),
                                            ident,
                                        ))
                                    }
                                }
                                StateModule::SharedState {
                                    value,
                                    ident: state_ident,
                                } => Some((
                                    Intermediate::Value(value.clone().to_string()),
                                    state_ident.clone(),
                                )),
                            }
                        }
                    },
                    _ => None,
                } {
                    actions_cache.insert(ident.clone(), inter);
                }
            }

            for segment in card.segments.clone() {
                match segment {
                    Segment::Raw { string } => {
                        buffer.push(string.clone());
                    }
                    Segment::Action { ident } => {
                        if let Some(inter) = actions_cache.get(&ident) {
                            if let Some(value) = inter.to_string() {
                                buffer.push(value);
                            }
                        }
                    }
                }
            }

            for action in card.actions.clone() {
                if (actions_cache.contains_key(&action.get_ident())) {
                    continue;
                } else {
                    match action {
                        Action::UpdateState {
                            ident,
                            state,
                            value,
                            add,
                            selector,
                        } => {
                            state_updates.insert(
                                ident.clone(),
                                StateUpdate {
                                    ident: ident.clone(),
                                    modifier: match value {
                                        Data::Integer { integer } => {
                                            if !add {
                                                integer * -1
                                            } else {
                                                integer
                                            }
                                        }
                                        _ => 0,
                                    },
                                    selector,
                                },
                            );
                        }
                        Action::Option {
                            ident,
                            display,
                            actions,
                        } => {
                            let mut updates = Vec::new();
                            for action_ident in actions {
                                if let Some(state_update) = state_updates.clone().get(&action_ident)
                                {
                                    updates.push(state_update.clone());
                                }
                            }
                            decisions.push(CardOption {
                                id: Uuid::new_v4(),
                                display,
                                updates,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        Some(CardResult {
            state_options: decisions,
            text: buffer.join(" "),
        })
    }
    pub fn change_deck(&mut self, bundle: DeckBundle) {
        self.game_state.change_deck(bundle);
    }

    pub fn reset_game_state(&mut self) {
        self.game_state.reset();
    }
}
