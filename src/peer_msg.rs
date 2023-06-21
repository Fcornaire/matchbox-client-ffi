use core::panic;

use matchbox_socket::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerMessage {
    peer_id: PeerId,
    message: String,
    peer_message_type: PeerMessageType,
}

impl PeerMessage {
    pub fn new(peer_id: PeerId, message: String, peer_message_type: PeerMessageType) -> Self {
        Self {
            peer_id,
            message,
            peer_message_type,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PeerMessageType {
    Ping,
    Pong,
    Archer,
    Greetings,
    RoundFinished,
}

impl From<&str> for PeerMessageType {
    fn from(s: &str) -> Self {
        match s {
            "Ping" => PeerMessageType::Ping,
            "Pong" => PeerMessageType::Pong,
            str if str.contains("Archer") => PeerMessageType::Archer,
            "Greetings" => PeerMessageType::Greetings,
            "RoundFinished" => PeerMessageType::RoundFinished,
            _ => panic!("Unknown peer type"),
        }
    }
}
