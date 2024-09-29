// Zasady ID's niestety jestem zbyt leniwy, żeby robić system macro
// za każdym razem jak się dodaje nowy typ ID do ENUM ID
// to trzeba dodać funkcje weryfikująca
// niestety ten system jest nie najlepszy, ale pozwala nam
// na to, że będziemy mieli pewność z ID jest specyficznego typu


use uuid::Uuid;

trait Id {}

pub type UserId = Uuid;

pub type SessionId = Uuid;

impl Id for Uuid {}
