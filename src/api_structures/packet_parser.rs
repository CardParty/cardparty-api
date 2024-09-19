use serde::{Deserialize, Serialize};
use serde_json::from_value;

#[derive(Serialize, Deserialize, Debug)]
struct Header{
    packet: String,
    timestamp: u64,
}
#[derive(Serialize, Deserialize, Debug)]
struct Packet{
    header: Header,
    data: String,
}
// #[derive(Deserialize, Debug)]
// struct UpdateData {
//     id: u32,
//     status: String,
// }
//
// #[derive(Deserialize, Debug)]
// struct CreateData {
//     name: String,
//     value: u64,
// }
#[derive(Deserialize, Debug)]
struct PlayerLeaveData {
    id: u32,
}
#[derive(Deserialize, Debug)]
struct PlayerFinishedData {
    id: u32,
}
#[derive(Deserialize, Debug)]
struct PlayerChoiceData {
    id: u32,
    choice: String,
}
enum DeserializedPacket {
    // Update(UpdateData),
    // Create(CreateData),
    PlayerLeave(PlayerLeaveData),
    PlayerFinished(PlayerFinishedData),
    PlayerChoice(PlayerChoiceData),
    Error,
}
pub fn deserialize_json(json: &str) -> DeserializedPacket {
    let packet: Packet = serde_json::from_str(json).unwrap();
    print!("Deserialized packet: {:?}", packet);
    match packet.header.packet.as_str() {
        // "update" => {
        //     let update_data: UpdateData = from_value(packet.data.parse().unwrap()).unwrap();
        //     println!("Deserialized as UpdateData: {:?}", update_data);
        // }
        // "create" => {
        //     let create_data: CreateData = from_value(packet.data.parse().unwrap()).unwrap();
        //     println!("Deserialized as CreateData: {:?}", create_data);
        // }
        "player_leave" => {
            let player_leave_data: PlayerLeaveData = from_value(packet.data.parse().unwrap()).unwrap();
            DeserializedPacket::PlayerLeave(player_leave_data)
        }

        "player_finished" => {
            let player_finished_data: PlayerFinishedData = from_value(packet.data.parse().unwrap()).unwrap();
            DeserializedPacket::PlayerFinished(player_finished_data)
        }
        "player_choice" => {
            let player_choice_data: PlayerChoiceData = from_value(packet.data.parse().unwrap()).unwrap();
            DeserializedPacket::PlayerChoice(player_choice_data)
        }
        //TODO: ZROBIĆ
        // "override_state"=>{
        //
        //
        // }
        _ => {
            DeserializedPacket::Error
        }

    }
}
