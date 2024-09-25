use actix::MailboxError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    card_game::deck::Card,
    card_game::deck::Deck,
    managers::game_manager::{GameState, StateOption},
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

pub enum PacketError {
    CipaChuj,
    DziwkaToTrojmiasto,
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
        new_state: GameState,
    },
    CardResult {
        state_options: Vec<StateOption>,
        text: String,
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
        chosen_state_id: Uuid,
    },
    PlayerDone {},
    CloseSession {},
}

pub enum PacketResponse {
    // API -> CLIENT
    UpdateStateOk,
    CardResultOk,
    FinishGameOk,

    // API <-> CLIENT
    AdminTokenOk,
    TestPacketWithStringOk { string: String },

    // API <- CLIENT
    SetDeckOk,
    PlayerLeftOk,
    PlayerDoneChoiseOk,
    CloseSessionOk,
    PlayerDoneOk,
}

pub fn deserialize_json(json: &str) -> Packet {
    let packet: Packet = serde_json::from_str(json).unwrap();
    packet
}

#[cfg(test)]

fn test_serialization() {}
