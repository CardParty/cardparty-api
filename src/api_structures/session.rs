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
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SessionError {}
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Player {
    pub username: String,
    pub id: UserId,
    is_host: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SessionState {
    Lobby,
    PreGame,
    Game,
    PostGame,
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
    pub connections: Vec<Addr<SessionConnection>>,
    pub host_id: Uuid,
    pub players: Vec<Player>,
    pub uncommited_players: Vec<Player>,
    pub admin_token: Uuid,
    pub game_manager: Option<GameManager>,
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
    ) -> (Addr<Self>, SessionId) {
        let id = Uuid::new_v4();
        let addr = Self {
            id,
            host_id,
            connections: Vec::new(),
            players: Vec::new(),
            uncommited_players: Vec::new(),
            admin_token: Uuid::new_v4(),
            game_manager: None,
            session_state: SessionState::Lobby,
            manager_addr,
            code,
        }
        .start();

        (addr, id)
    }
    // pub fn add_game_manager(&mut self, deck_bundle: DeckBundle) {
    //     self.game_manager = Some(GameManager::init(deck_bundle));
    // }
    // pub fn get_game_manager(self) -> GameManager {
    //     self.game_manager.unwrap()
    // }
    // pub fn advance_state(&mut self) {
    //     match self.session_state {
    //         SessionState::Lobby => self.session_state = SessionState::PreGame,
    //         SessionState::PreGame => self.session_state = SessionState::Game,
    //         SessionState::Game => self.session_state = SessionState::PostGame,
    //         SessionState::PostGame => self.session_state = SessionState::Lobby,
    //     }
    // }

    // pub fn get_state(self) -> SessionState {
    //     self.session_state
    // }
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
        self.players
            .push(Player::new(msg.id, msg.username.clone(), msg.is_host));
        if self.players.len() == 1 {
            let conn = SessionConnection::new(msg.id, msg.session_addr, true);
            let players = self
                .players
                .iter()
                .map(|x| x.username.clone())
                .collect::<Vec<String>>();
            for conn in self.connections.clone() {
                if let Some(game_manager) = self.game_manager.as_mut() {
                    if self.uncommited_players.is_empty() {
                        log::info!("Sending player update emptry ucp");
                        conn.do_send(PlayerUpdate(players.clone(), game_manager.bundle_state()));
                    } else {
                        for player in self.uncommited_players.drain(..) {
                            log::info!("adding from ucp");
                            conn.do_send(PlayerUpdate(players.clone(), game_manager.bundle_state()));
                            game_manager.add_player(player.id, player.username.clone(), player.is_host);
                        }
                    }

                } else {
                    self.uncommited_players
                      .push(Player::new(msg.id, msg.username.clone(), msg.is_host));
                }

                log::info!("Uncommited Pl : {:#?}", self.uncommited_players);
                log::info!("Players: {:#?}", self.players);
            }
            Ok(conn)
        } else {
            let conn = SessionConnection::new(msg.id, msg.session_addr, false);
            let players = self
                .players
                .iter()
                .map(|x| x.username.clone())
                .collect::<Vec<String>>();
            for conn in self.connections.clone() {
                if let Some(game_manager) = self.game_manager.as_mut() {
                    if self.uncommited_players.is_empty() {
                        log::info!("Sending player update emptry ucp");
                        conn.do_send(PlayerUpdate(players.clone(), game_manager.bundle_state()));
                    } else {
                        for player in self.uncommited_players.drain(..) {
                            log::info!("adding from ucp");
                            conn.do_send(PlayerUpdate(players.clone(), game_manager.bundle_state()));
                            game_manager.add_player(player.id, player.username.clone(), player.is_host);
                        }
                    }

                } else {
                    self.uncommited_players
                      .push(Player::new(msg.id, msg.username.clone(), msg.is_host));
                }

                log::info!("Uncommited Pl : {:#?}", self.uncommited_players);
                log::info!("Players: {:#?}", self.players);
            }
            Ok(conn)
        }
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
        for conn in &self.connections {
            conn.do_send(SendToClient(msg.0.clone()))
        }
    }
}

impl Handler<AddConnection> for Session {
    type Result = ();

    fn handle(&mut self, msg: AddConnection, _ctx: &mut Self::Context) -> Self::Result {
        log::info!("Adding connection: {:#?}", msg.0);
        self.connections.push(msg.0);
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
                    if let Some(game_manager) = self.game_manager.as_mut() {
                    game_manager.change_deck(deck.into_bundle());
                    Ok(PacketResponse::SetDeckOk { bundle: game_manager.bundle_state() })
                } else {
                    self.game_manager = Some(GameManager::init(deck.into_bundle()));
                        Ok(PacketResponse::SetDeckOk { bundle: self.game_manager.clone().unwrap().bundle_state() })
                } }
            Packet::PlayerLeft { id } => {
                if let Some(game_manager) = self.game_manager.as_mut() {
                game_manager.remove_player(id);
                self.players.retain(|x| x.id != id);
                if self.players.len() == 0 {
                    self.session_state = SessionState::Lobby;

                    for conn in &self.connections {
                        conn.do_send(CloseSessionConnection);
                    }
                    self.manager_addr.do_send(CloseSession(self.id.clone()));
                    Ok(PacketResponse::CloseSessionOk)
                } else {
                    self.host_id = self.players[0].id;
                    Ok(PacketResponse::PlayerLeftOk { bundle: game_manager.bundle_state() } )
                }
            } else {
                Err(PacketError::GameManagerError)
            }}
            Packet::PlayerDoneChoise { chosen } => {
                log::info!("Player done choise: {:#?}", chosen);
                if let Some(game_manager) = self.game_manager.as_mut() {
                    game_manager.resolve_state(chosen);
                    let card = game_manager.get_next_card().unwrap();
                    let bundle = game_manager.bundle_state();

                    for conn in self.connections.clone() {
                        conn.do_send(SendPacket(Packet::CardResult { card: card.clone(), bundle: bundle.clone() }));
                    }

                Ok(PacketResponse::CardResultOk { card: card.clone(), bundle: bundle.clone() })
                } else {
                    Err(PacketError::GameManagerError)
                }
            }
            Packet::PlayerDone { .. } => {
                log::info!("Player done");
                if let Some(game_manager) = self.game_manager.as_mut() {
                    let card = game_manager.get_next_card().unwrap();
                    let bundle = game_manager.bundle_state();

                    for conn in self.connections.clone() {
                        conn.do_send(SendPacket(Packet::CardResult { card: card.clone(), bundle: bundle.clone() }));
                    }

                    Ok(PacketResponse::CardResultOk { card: card.clone(), bundle: bundle.clone() })
                } else {
                    Err(PacketError::GameManagerError)
                }
            }
            Packet::CloseSession { .. } => {
                log::info!("Closing session: {:#?}", self.id);
                for conn in self.connections.clone() {
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
