use std::cell::{Cell, RefCell};
use std::rc::Rc;
use super::managers::session_manager::SessionManager;
use super::messages::{
    AddConnection, AddPlayer, CloseSession, CloseSessionConnection, GetHostId, GetSessionId,
    PlayerUpdate, SendPacket, SendToClient, VerifyExistence,
};
use super::packet_parser::{Packet, PacketError, PacketResponse};
use super::session_connection::SessionConnection;
use crate::api_structures::id::*;
use crate::api_structures::managers::game_manager::GameManager;
use crate::api_structures::messages::BroadcastMessage;
use crate::api_structures::messages::TestMessage;
use actix::{Actor, Addr, Context, Handler};
use chrono::format::Item;
use rand::prelude::*;

use serde::{Deserialize, Serialize};

fn generate_random_string(length: usize) -> String {
    // Define the character set: lowercase a-z and digits 0-9
    let chars = "abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = thread_rng();

    // Generate a random string
    (0..length)
        .map(|_| {
            let index = rng.gen_range(0..chars.len()); // Generate a random index
            chars.chars().nth(index).unwrap() // Get the character at that index
        })
        .collect() // Collect into a String
}

use uuid::Uuid;
use crate::api_structures::card_game::deck::{Deck, Selector};
use crate::api_structures::session::SessionState::Game;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Players {
    pub players: Vec<Player>,
    pub idx: Cell<usize>,
}


impl Players {
    pub fn new() -> Self {
        Self { players: Vec::new(), idx: Cell::new(0)  }
    }
    pub fn add_player(&mut self, player: Player) {
        self.players.push(player);
        self.players.shuffle(&mut thread_rng());
    }

    pub fn remove_player(&mut self, id: UserId) {
        self.players.retain(|x| x.id != id);
    }
    pub fn get_players(&self) -> Vec<String> {
        self.players.iter().map(|x| x.username.clone()).collect()
    }

    pub fn consume(&self) {
        self.idx.set(self.idx.get() + 1);
    }

    pub fn get_player(&self, selector: Selector) -> &Player {
        match selector {
            Selector::Current => &self.players[self.idx.get()],
            Selector::Next => &self.players[(self.idx.get() + 1) % self.players.len()],
            Selector::Previous => &self.players[(self.idx.get() - 1) % self.players.len()],
            Selector::Random => &self.players[thread_rng().gen_range(0..self.players.len())],
            Selector::None => self.players.first().unwrap(),
        }
    }

    pub fn clone_player(&self, selector: Selector) -> Player {
        match selector {
            Selector::Current => self.players[self.idx.get()].clone(),
            Selector::Next => self.players[(self.idx.get()  + 1) % self.players.len()].clone(),
            Selector::Previous => self.players[(self.idx.get() - 1) % self.players.len()].clone(),
            Selector::Random => self.players[thread_rng().gen_range(0..self.players.len())].clone(),
            Selector::None => self.players.first().unwrap().clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Connections {
    pub connections: Vec<Addr<SessionConnection>>,
}

impl Connections {
    pub fn new() -> Self {
        Self { connections: Vec::new() }
    }

    pub fn add_connection(&mut self, connection: Addr<SessionConnection>) {
        self.connections.push(connection);
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Player {
    pub username: String,
    pub id: UserId,
    is_host: bool,
}

impl Player {
    pub fn new(id: UserId, username: String, is_host: bool) -> Self {
        Self {
            id,
            username,
            is_host,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SessionError {}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SessionState {
    Lobby,
    PreGame,
    Game,
    PostGame,
}


#[derive(PartialEq, Eq, Hash, Clone)]
pub struct SessionCode {
    pub code: String,
}

impl SessionCode {
    pub fn gen() -> Self {
        Self {
            code: generate_random_string(6),
        }
    }
    pub fn regen(&mut self) {
        self.code = generate_random_string(6)
    }

    pub fn from(str: String) -> Self {
        Self { code: str }
    }
}

#[derive(Clone)]
pub struct Session {
    pub id: SessionId,
    pub connections: Connections,
    pub host_id: Uuid,
    pub players: Rc<RefCell<Players>>,
    pub admin_token: Uuid,
    pub game_manager: GameManager,
    pub session_state: SessionState,
    pub manager_addr: Addr<SessionManager>,
    pub code: SessionCode,
}

impl Actor for Session {
    type Context = Context<Self>;
}

impl Session {
    pub async fn init(
        host_id: UserId,
        _username: String,
        manager_addr: Addr<SessionManager>,
        code: SessionCode,
        deck: Deck,
    ) -> (Addr<Self>, SessionId) {
        let id = Uuid::new_v4();
        let plrs = Rc::new(RefCell::new(Players::new()));
        let plrs_clone = Rc::clone(&plrs);

        let addr = Self {
            id,
            host_id,
            connections: Connections::new(),
            players: plrs,
            admin_token: Uuid::new_v4(),
            game_manager: GameManager::init(deck.into_bundle(), plrs_clone),
            session_state: SessionState::Lobby,
            manager_addr,
            code,
        }
          .start();

        (addr, id)
    }
}

impl Handler<TestMessage> for Session {
    type Result = ();
    fn handle(&mut self, _msg: TestMessage, _ctx: &mut Self::Context) -> Self::Result {}
}

impl Handler<VerifyExistence> for Session {
    type Result = bool;
    fn handle(&mut self, msg: VerifyExistence, _ctx: &mut Self::Context) -> Self::Result {
        self.id == msg.0
    }
}

impl Handler<GetHostId> for Session {
    type Result = String;
    fn handle(&mut self, _msg: GetHostId, _ctx: &mut Self::Context) -> Self::Result {
        self.host_id.to_string()
    }
}

impl Handler<AddPlayer> for Session {
    type Result = Result<SessionConnection, SessionError>;
    fn handle(&mut self, msg: AddPlayer, _ctx: &mut Self::Context) -> Self::Result {
        log::info!("Adding player: {:#?} to session: {:#?}", msg, self.id);
        let player = Player::new(msg.id, msg.username, msg.is_host);
        self.players.borrow_mut().add_player(player);

        self.game_manager.regen();

        let connection = SessionConnection::new(msg.id, msg.session_addr, msg.is_host);

        for conn in &self.connections.connections {
            conn.do_send(PlayerUpdate(self.players.borrow().get_players(), self.game_manager.bundle_state()));
        }

        Ok(connection)
    }
}

impl Handler<GetSessionId> for Session {
    type Result = String;
    fn handle(&mut self, _msg: GetSessionId, _ctx: &mut Self::Context) -> Self::Result {
        self.id.to_string()
    }
}
impl Handler<BroadcastMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _ctx: &mut Self::Context) -> Self::Result {
        for conn in &self.connections.connections {
            conn.do_send(SendToClient(msg.0.clone()))
        }
    }
}

impl Handler<AddConnection> for Session {
    type Result = ();

    fn handle(&mut self, msg: AddConnection, _ctx: &mut Self::Context) -> Self::Result {
        log::info!("Adding connection: {:#?}", msg.0);
        self.connections.add_connection(msg.0);
    }
}



impl Handler<SendPacket> for Session {
    type Result = Result<PacketResponse, PacketError>;

    fn handle(&mut self, msg: SendPacket, _ctx: &mut Self::Context) -> Self::Result {
        match msg.0 {
            Packet::TestError {  } => {
                Err(PacketError::CipaChuj)
            }
            Packet::TestPacketWithString { string } => {
                Ok(PacketResponse::TestPacketWithStringOk { string })
            }
            Packet::SetDeck { deck } => {
                log::info!("Setting deck: {:#?}", deck);
                self.game_manager.change_deck(deck.into_bundle());
                Ok(PacketResponse::SetDeckOk { bundle: self.game_manager.bundle_state() })
            }
            Packet::PlayerLeft { id } => {
                self.game_manager.remove_player(id);
                self.players.borrow_mut().players.retain(|x| x.id != id);
                if self.players.borrow().players.is_empty() {
                    self.session_state = SessionState::Lobby;

                    for conn in &self.connections.connections {
                        conn.do_send(CloseSessionConnection);
                    }
                    self.manager_addr.do_send(CloseSession(self.id.clone()));
                    Ok(PacketResponse::CloseSessionOk)
                } else {
                    self.host_id = self.players.borrow().players[0].id;
                    Ok(PacketResponse::PlayerLeftOk { bundle: self.game_manager.bundle_state() })
                }
            }
            Packet::PlayerDoneChoise { chosen } => {
                log::info!("Player done choise: {:#?}", chosen);
                self.game_manager.resolve_state(chosen);
                let card = self.game_manager.get_next_card().unwrap();
                let bundle = self.game_manager.bundle_state();

                for conn in self.connections.connections.clone() {
                    conn.do_send(SendPacket(Packet::CardResult { card: card.clone(), bundle: bundle.clone() }));
                }

                Ok(PacketResponse::CardResultOk { card: card.clone(), bundle: bundle.clone() })
            }
            Packet::PlayerDone { .. } => {
                log::info!("Player done");
                let card = self.game_manager.get_next_card().unwrap();
                let bundle = self.game_manager.bundle_state();

                for conn in self.connections.connections.clone() {
                    conn.do_send(SendPacket(Packet::CardResult { card: card.clone(), bundle: bundle.clone() }));
                }

                Ok(PacketResponse::CardResultOk { card: card.clone(), bundle: bundle.clone() })
            }
            Packet::CloseSession { .. } => {
                log::info!("Closing session: {:#?}", self.id);
                for conn in self.connections.connections.clone() {
                    conn.do_send(CloseSessionConnection);
                }
                self.manager_addr.do_send(CloseSession(self.id.clone()));
                Ok(PacketResponse::CloseSessionOk)
            }

            _ => {
                Err(PacketError::CipaChuj)
            }
        }
    }
}
