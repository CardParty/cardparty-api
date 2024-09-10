use uuid::Uuid;

use crate::api_structures::{id::UserId, session::Player};

use super::deck::Deck;

/*next_player
in gameState change the current player to the next one

end_game
shutdown all connections

update_state
client will provide state uuid and new value

player_leave
send to sever uuid of left player

init_deck
client sends deck json to api

init_state
*/

enum PacketOperation {
    NextPlayer,
    EndGame,
    UpdateState { state_id: Uuid, value: String },
    PlayerLeave { player_id: UserId },
}

//TODO coock this shit
struct State {
    uuid: String,
    value: String,
}
struct GameState {
    current_player: usize,
    states: Vec<State>,
}

struct Game {
    deck: Deck,
    players: Vec<Player>,
}
