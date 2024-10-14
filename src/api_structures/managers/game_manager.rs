use std::cell::RefCell;
use crate::api_structures::card_game::deck::{Action, Card, Data, DeckBundle, RenderedScoreBoard, Segment, StateModule, TextElement, TextInfo, Value};
use crate::api_structures::card_game::deck::{ScoreBoard, Selector};
use crate::api_structures::id;
use crate::api_structures::session::{Player, Players};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBundle {
    score_board: RenderedScoreBoard,
    current_idx: usize,
    states: Vec<StateModule> }

impl Default for GameBundle {
    fn default() -> Self {
        Self {
            score_board: RenderedScoreBoard::default(),
            current_idx: 69,
            states: Vec::new(),
        }
    }
}

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
    pub text: TextInfo,
}

#[derive(Clone, Debug)]
pub struct GameState {
    players: Rc<RefCell<Players>>,
    tables: HashMap<String, Vec<Value>>,
    states: HashMap<String, StateModule>,
    cards: Vec<Card>,
    card_count: usize,
    current: usize,
    score_state: ScoreBoard,
}

impl GameState {
    pub fn new(bundle: DeckBundle, players: Rc<RefCell<Players>>) -> Self {
        Self {
            players,
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

    pub fn bundle_state(&self) -> GameBundle {

        if let Ok(score_board) = self.score_state.generate_scoreboard(self.states.clone(), self.players.borrow().players.clone()) {
            GameBundle {
                score_board,
                current_idx: self.current,
                states: self.states.clone().into_values().filter_map(|v| match v {
                    StateModule::SharedState { .. } => Some(v),
                    _ => None,
                }).collect()
            }
        } else {
            panic!("Failed to generate scoreboard")
        }
    }


}

#[derive(Clone, Debug)]
pub struct GameManager {
    rng: ThreadRng,
    players: Rc<RefCell<Players>>,
    game_state: GameState,
    awaited_states: HashMap<Uuid, CardOption>,
}

impl GameManager {
    pub fn init(bundle: DeckBundle, players: Rc<RefCell<Players>>) -> Self {
        Self {
            rng: thread_rng(),
            game_state: GameState::new(bundle.clone(), players.clone()),
            awaited_states: HashMap::new(),
            players,
        }
    }
    pub fn regen(&mut self) {
        let players = self.players.clone();
        self.regen_states(&*players.borrow());
    }

    pub fn resolve_state(&mut self, id: Uuid) {
        log::info!("Resolving state {:#?}", id);
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
                        let players = &*self.players.borrow();
                        let player = players.get_player(update.selector);
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
        self.game_state.cards.shuffle(&mut self.rng);
    }

    pub fn regen_states(&mut self, players: &Players) {
        log::info!("Regenerating states");
        let player_ids: HashSet<_> = players.players.iter().map(|p| p.id).collect();
        let prnt_state = self.game_state.states.clone();
        for (_, state) in self.game_state.states.iter_mut() {

            if let StateModule::IndividualState {
                constructor_value,
                map,
            } = state {
                log::info!("Regenerating state {:#?}", prnt_state);
                map.retain(|player_id, _| player_ids.contains(player_id));


                for player_id in &player_ids {
                    map.entry(*player_id).or_insert(*constructor_value);
                }
            }
        }
    }

    pub fn remove_player(&mut self, id: Uuid) {
        log::info!("Removing player {:#?}", id);
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

    pub fn get_next_card(&mut self) -> Option<CardResult> {
        log::info!("Getting next card");
        let mut buffer: Vec<TextElement> = Vec::new();
        let mut decisions: Vec<CardOption> = Vec::new();

        let mut actions_cache: HashMap<String, Intermediate> = HashMap::new();
        let mut state_updates: HashMap<String, StateUpdate> = HashMap::new();
        
        let mut bg = String::new();
        let mut general_text = String::new();

        if let Some(card) = self.game_state.cards.choose(&mut self.rng) {
            bg = card.bg.clone();
            general_text = card.general_text.clone();
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
                                Intermediate::Value("Error".to_string()),
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
                            let players = self.players.borrow();
                            let player = players.get_player(selector);
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
                        buffer.push(string);
                    }
                    Segment::Action { ident } => {
                        if let Some(inter) = actions_cache.get(&ident) {
                            if let Some(value) = inter.to_string() {
                                buffer.push(TextElement::span {content: value, text_color:"white".to_string(), bold:false});
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

        let players = self.players.borrow();
        players.consume();

        log::info!("Returning card text: {:#?}", buffer);
        log::info!("Returning card decisions: {:#?}", decisions);
        Some(CardResult {
            state_options: decisions,
            text: TextInfo {
                bg,
                general_text,
                text: buffer,
            },
        })
    }
    pub fn change_deck(&mut self, bundle: DeckBundle) {
        log::info!("Changing deck: {:#?}", bundle);
        self.game_state.change_deck(bundle);
    }

    pub fn reset_game_state(&mut self) {
        log::info!("Resetting game state {:#?}", self.game_state);
        self.game_state.reset();
    }

    pub fn bundle_state(&self) -> GameBundle {
        let bundle = self.game_state.bundle_state();
        log::info!("Bundling state {:#?}", &bundle);
        bundle
    }
}
