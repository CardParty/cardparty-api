use serde::{Deserialize, Serialize};
use serde_json::from_value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct Header{
    packet: String,
    timestamp: u64,
}
#[derive(Serialize, Deserialize, Debug)]

pub enum Data{
    PlayerLeaveData {
        id: String,
    },
    PlayerFinishedData {
        id: String,
    },
    PlayerChoiceData {
        id: String,
    },
    Error,

}

impl Data {
    fn player_leave(p0: Data) -> Data {
        todo!()
    }
    fn player_finished(p0: Data) -> Data {
        todo!()
    }
    fn player_choice(p0: Data) -> Data {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Packet{
    header: Header,
    data: Data,
}

pub fn deserialize_json(json: &str) -> Data {
    let packet: Packet = serde_json::from_str(json).unwrap();
    print!("Deserialized packet: {:?}", packet);
    match packet.header.packet.as_str() {
        "player_leave" => {
            let player_leave_data=Data::PlayerLeaveData {
                id: packet.data.id.parse().unwrap()
            };
            Data::player_leave(player_leave_data)
        }

        "player_finished" => {
            let player_finished_data=Data::PlayerFinishedData {
                id: packet.data.id.parse().unwrap()
            };
            Data::player_finished(player_finished_data)
        }
        "player_choice" => {
            let player_choice_data=Data::PlayerChoiceData {
                id: packet.data.id.parse().unwrap(),
            };
            Data::player_choice(player_choice_data)
        }
        //TODO: ZROBIĆ
        // "override_state"=>{
        //
        //
        // }
        _ => {
            Data::Error
        }

    }
}
