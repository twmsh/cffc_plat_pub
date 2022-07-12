use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
};

use actix::prelude::*;
use log::{debug, error};
use rand::{self, Rng, rngs::ThreadRng};

use super::worker::WsWorker;
use super::{DEFAULT_ROOM, SessionConnect, SessionDisconnect, WsMessage, DeliverMessage};
use crate::services::ws::RegisterMessage;

pub struct WsAgent {
    sessions: HashMap<usize, Recipient<WsMessage>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    session_count: AtomicUsize,

    //
    pub addr: Addr<WsWorker>,
}


impl WsAgent {
    pub fn new(addr: Addr<WsWorker>) -> Self {

        // default room
        let mut rooms = HashMap::new();
        rooms.insert(DEFAULT_ROOM.to_string(), HashSet::new());

        WsAgent {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            session_count: AtomicUsize::new(0),
            addr
        }
    }
}

impl WsAgent {
    fn broadcast_message(&self, room: &str, message: &str) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if let Some(addr) = self.sessions.get(id) {
                    debug!("WS, do_send, id:{}", id);
                    let _ = addr.do_send(WsMessage(message.to_owned()));
                }
            }
        }
    }

    fn deliver_message(&self, id: usize, message: &str) {
        if let Some(addr) = self.sessions.get(&id) {
            let _ = addr.do_send(WsMessage(message.to_string()));
        } else {
            error!("error, WS, session:{} not found", id);
        }
    }
}


impl Actor for WsAgent {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.addr.do_send(RegisterMessage {
            addr: addr.recipient(),
        });
    }
}


/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<SessionConnect> for WsAgent {
    type Result = usize;

    fn handle(&mut self, msg: SessionConnect, _: &mut Context<Self>) -> Self::Result {
        debug!("WS, Someone connected.");

        let session_addr = msg.addr.clone();
        let session_room = msg.room.clone();

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to Main room
        self.rooms
            .entry(msg.room.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        let count = self.session_count.fetch_add(1, Ordering::SeqCst);
        debug!("WS, id:{} connected. total: {}", id, count + 1);

        // 向上层发消息
        self.addr.do_send(SessionConnect {
            addr: session_addr,
            room: session_room,
            id,
        });

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<SessionDisconnect> for WsAgent {
    type Result = ();

    fn handle(&mut self, msg: SessionDisconnect, _: &mut Context<Self>) {
        let count = self.session_count.fetch_sub(1, Ordering::SeqCst);
        debug!("WS, id:{} disconnected. total: {}", msg.id, count - 1);

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (_name, sessions) in &mut self.rooms {
                let _ = sessions.remove(&msg.id);
            }
        }
    }
}



/// Handler for broadcast message.
impl Handler<DeliverMessage> for WsAgent {
    type Result = ();

    fn handle(&mut self, msg: DeliverMessage, _: &mut Context<Self>) {
        if msg.id == 0 {
            self.broadcast_message(&msg.room, &msg.msg);
        } else {
            self.deliver_message(msg.id, &msg.msg);
        }
    }
}
