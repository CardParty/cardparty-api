use crate::api_structures::{messages::*, packet_parser::deserialize_json};

use super::session::Session;
use super::{id::*, packet_parser::PacketResponse};
use actix::{Actor, Addr, Handler, StreamHandler};
use actix_web_actors::ws;
use futures::executor::block_on;
use uuid::Uuid;
use crate::api_structures::managers::game_manager::GameBundle;
use crate::api_structures::packet_parser::{Packet, PacketError};

pub struct SessionConnection {
    session: Addr<Session>,
    user_id: UserId,
    id: Uuid,
    is_admin: bool,
}

impl SessionConnection {
    pub fn new(user_id: UserId, session: Addr<Session>, is_admin: bool) -> Self {
        Self {
            user_id,
            session,
            id: Uuid::new_v4(),
            is_admin,
        }
    }
}

impl Actor for SessionConnection {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SessionConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
               
                if  text == "dump" {
                    self.session.do_send(Dump());
                    return;
                }
                
                log::info!("Received text: {:?}", text);
                let packet = deserialize_json(&text);
                log::info!("Received packet: {:?}", packet);
                let response = block_on(async { self.session.send(SendPacket(packet)).await? });

                
                
                match response {
                    Ok(resp) => {
                        if let PacketResponse::Unit = resp {
                            return;
                        }
                        log::info!("Response: {:?}", resp);
                        ctx.text(serde_json::to_string(&resp).unwrap());
                        match resp {
                            PacketResponse::CloseSessionOk => {
                                ()
                            }
                            _ => {
                                ctx.text(serde_json::to_string(&PacketResponse::UpdateStateOk { bundle: resp.get_bundle().unwrap() }).unwrap());
                            }
                        }
                    }
                    Err(err) => {
                        ctx.text(serde_json::to_string(&err).expect("Failed to serialize Error"));
                    }
                }
            }
            _ => (),
        }
    }
}

impl Handler<TestMessage> for SessionConnection {
    type Result = ();

    fn handle(&mut self, msg: TestMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Handler<SendToClient> for SessionConnection {
    type Result = ();
    fn handle(&mut self, msg: SendToClient, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

impl Handler<ConnectWithSession> for SessionConnection {
    type Result = ();

    fn handle(&mut self, msg: ConnectWithSession, _ctx: &mut Self::Context) -> Self::Result {
        self.session.do_send(AddConnection(msg.0))
    }
}

impl Handler<CloseSessionConnection> for SessionConnection {
    type Result = ();

    fn handle(&mut self, _msg: CloseSessionConnection, ctx: &mut Self::Context) -> Self::Result {
        ctx.close(None);
    }
}

impl Handler<PlayerUpdate> for SessionConnection {
    type Result = ();

    fn handle(&mut self, msg: PlayerUpdate, ctx: &mut Self::Context) -> Self::Result {

        let resp = PacketResponse::PlayersUpdateOk { players: msg.0, bundle: GameBundle::default() };
        ctx.text(
            serde_json::to_string(&resp ).unwrap(),
        );
    }
}

impl Handler<SendPacket> for SessionConnection {
    type Result = Result<PacketResponse, PacketError>;

    fn handle(&mut self, msg: SendPacket, ctx: &mut Self::Context) -> Self::Result {
        
       let resp = match msg.0 {
          Packet::CardResult { card, bundle } => Ok(PacketResponse::StartGameOk),
           Packet::StartGame {} => Ok(PacketResponse::UpdateStateOk { bundle: GameBundle::default() }),
           Packet::FinishGame {} => Ok(PacketResponse::FinishGameOk),
          _ => {
              log::error!("Unknown packet: {:?}", msg.0);
              Err(PacketError::Errorito)
          }
       };

       match resp {
            Ok(resp) => {ctx.text(serde_json::to_string(&resp).unwrap()); Ok(PacketResponse::AdminTokenOk)},
            Err(err) => Err(err),
       }
    }
}
