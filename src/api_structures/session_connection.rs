use crate::api_structures::{messages::*, packet_parser::deserialize_json};

use super::session::Session;
use super::{id::*, packet_parser::PacketResponse};
use actix::{Actor, Addr, Handler, StreamHandler};
use actix_web_actors::ws;
use futures::executor::block_on;
use uuid::Uuid;

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
                let packet = deserialize_json(&text);
                let response = block_on(async { self.session.send(SendPacket(packet)).await? });

                match response {
                    Ok(resp) => match resp {
                        PacketResponse::TestPacketWithStringOk { string } => {
                            ctx.text(string);
                        }
                        _ => {}
                    },
                    Err(err) => {
                        ctx.text("ajaj cos sie wyjebało");
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

    fn handle(&mut self, msg: CloseSessionConnection, ctx: &mut Self::Context) -> Self::Result {
        ctx.close(None);
    }
}
