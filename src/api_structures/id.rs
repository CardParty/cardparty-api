// Zasady ID's niestety jestem zbyt leniwy zeby robic system macro
// za kazdym razem jak sie dodaje nowy typ ID do ENUM ID
// to trzebda dodac funkcje weryfikujaca
// niestety ten system jest nienajlepszy ale pozwala nam
// na to ze bendziemy mielei pewnosc ze ID jest specyficznego typu

// system akutalnie dla testow jest na bazie I32
// TODO:
// Zmenic system id na UUID4 dla uniwersalonsci

use serde::de::value;

pub enum Id {
    UserId(i32),
    SessionId(i32),
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
}
