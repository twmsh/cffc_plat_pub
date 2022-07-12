pub mod session;
pub mod agent;
pub mod worker;

use actix::prelude::*;
use crate::queue_item::QI;

const DEFAULT_ROOM: &str = "";


#[derive(Message)]
#[rtype(result = "()")]
pub struct QiMessage(pub Vec<QI>);

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct SessionConnect {
    pub addr: Recipient<WsMessage>,
    pub room: String,
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SessionDisconnect {
    pub id: usize,
}

/// worker发送给agent
/// 群发 (id = 0)
/// 指定 session id
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeliverMessage {
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
    /// client id, id=0 broadcast
    pub id: usize,
}

/// agent 发送给 worker
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterMessage {
    pub addr: Recipient<DeliverMessage>,
}
