use actix::MailboxError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api_structures::managers::game_manager::{CardResult, GameBundle};
use super::{
    card_game::deck::Deck,
    managers::game_manager::CardOption,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    name: String,
    values: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Value {
    value: String,
    tags: Vec<String>,
}

pub struct State {
    value: i32,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "error")]
pub enum PacketError {
    CipaChuj,
    GameManagerError,
    #[serde(skip)]
    MailboxError(MailboxError),
}

impl From<MailboxError> for PacketError {
    fn from(value: MailboxError) -> Self {
        PacketError::MailboxError(value)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "packet")]
pub enum Packet {
    // API -> CLIENT
    UpdateState {
        new_state: GameBundle,
    },
    PlayersUpdate {},
    CardResult {
        state_options: Vec<CardOption>,
        display: String,
    },
    FinishGame {},

    // API <-> CLIENT
    AdminToken {
        token: Uuid,
    },
    TestError {},
    TestPacketWithString {
        string: String,
    },

    // API <- CLIENT
    SetDeck {
        deck: Deck,
    },
    PlayerLeft {
        id: Uuid,
    },
    PlayerDoneChoise {
        chosen: Uuid,
    },
    PlayerDone {},
    CloseSession {},
    GetPlayers {},
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "packet")]
pub enum PacketResponse {
    // API -> CLIENT
    UpdateStateOk { bundle: GameBundle },
    CardResultOk { card: CardResult, #[serde(skip)] bundle: GameBundle },
    FinishGameOk,

    // API <-> CLIENT
    AdminTokenOk,
    TestPacketWithStringOk { string: String },

    // API <- CLIENT
    SetDeckOk { #[serde(skip)] bundle: GameBundle },
    PlayerLeftOk { #[serde(skip)] bundle: GameBundle },
    PlayerDoneChoiseOk { #[serde(skip)] bundle: GameBundle },
    CloseSessionOk,
    PlayerDoneOk { #[serde(skip)] bundle: GameBundle },
    GetPlayersOk { players: Vec<String>,  #[serde(skip)] bundle: GameBundle },
    PlayersUpdateOk { players: Vec<String>, #[serde(skip)] bundle: GameBundle },
}

impl PacketResponse {
    pub fn get_bundle(&self) -> Option<GameBundle> {
        match self {
            PacketResponse::SetDeckOk { bundle } => Some(bundle.clone()),
            PacketResponse::PlayerLeftOk { bundle } => Some(bundle.clone()),
            PacketResponse::PlayerDoneChoiseOk { bundle } => Some(bundle.clone()),
            PacketResponse::PlayerDoneOk { bundle } => Some(bundle.clone()),
            PacketResponse::GetPlayersOk { bundle, .. } => Some(bundle.clone()),
            PacketResponse::PlayersUpdateOk { bundle, .. } => Some(bundle.clone()),
            PacketResponse::UpdateStateOk { bundle } => Some(bundle.clone()),
            PacketResponse::CardResultOk { bundle, .. } => Some(bundle.clone()),
            _ => None,
        }
    }
}

pub fn deserialize_json(json: &str) -> Packet {
    let packet: Packet = serde_json::from_str(json).unwrap();
    packet
}

#[cfg(test)]

fn test_serialization() {}
