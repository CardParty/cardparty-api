// Zasady ID's niestety jestem zbyt leniwy zeby robic system macro
// za kazdym razem jak sie dodaje nowy typ ID do ENUM ID
// to trzebda dodac funkcje weryfikujaca
// niestety ten system jest nienajlepszy ale pozwala nam
// na to ze bendziemy mielei pewnosc ze ID jest specyficznego typu

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, PartialEq)]

pub enum Id {
    UserId(Uuid),
    SessionId(Uuid),
}

impl Id {
    pub fn verify_user_id(self) -> Option<Self> {
        match self {
            Id::UserId(_) => Some(self),
            Id::SessionId(_) => None,
        }
    }
    pub fn verify_session_id(self) -> Option<Self> {
        match self {
            Id::UserId(_) => None,
            Id::SessionId(_) => Some(self),
        }
    }

    pub fn to_string(self) -> String {
        match self {
            Id::SessionId(id) => id.to_string(),
            Id::UserId(id) => id.to_string(),
        }
    }
}
