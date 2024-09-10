use actix::Addr;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api_structures::session::{Player, SessionConnection};
#[derive(Serialize, Deserialize, Debug)]
pub struct Deck {}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketEvent {
    pub meta_data: MetaData,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketEventBuilder {
    pub meta_data: Option<MetaData>,
    pub data: Option<Data>,
}

impl WebsocketEvent {
    pub fn configure() -> WebsocketEventBuilder {
        WebsocketEventBuilder {
            meta_data: None,
            data: None,
        }
    }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaData {
    pub event_type: EventType,
    pub timestamp: String,
    pub event_id: Uuid,
    #[serde(skip)]
    pub adressor: Option<Addr<SessionConnection>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Data {
    Deck(Deck),
    Players(Vec<Player>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventType {
    Responding,
    UpdateState,
    ParseCard,
    NextPlayer,
    Close,
    PlayerJoined,
    ExecuteCommand,
    PlayerLeft,
}
