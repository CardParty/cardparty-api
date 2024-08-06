// Zasady ID's niestety jestem zbyt leniwy zeby robic system macro
// za kazdym razem jak sie dodaje nowy typ ID do ENUM ID
// to trzebda dodac funkcje weryfikujaca
// niestety ten system jest nienajlepszy ale pozwala nam
// na to ze bendziemy mielei pewnosc ze ID jest specyficznego typu

use serde::{Deserialize, Serialize};
use uuid::Uuid;

trait Id {}

pub type UserId = Uuid;
pub type SessionId = Uuid;

impl Id for Uuid {}
