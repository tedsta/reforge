use std::collections::HashMap;

use net::{ClientId, ServerSlot, Client, Joined, ReceivedPacket, OutPacket};
use ship::Ship;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server

pub struct ServerBattleState {
    slot: Box<ServerSlot>,
    ships: HashMap<ClientId, Ship>,
}

impl ServerBattleState {
    pub fn new(slot: Box<ServerSlot>, ships: HashMap<ClientId, Ship>) -> ServerBattleState {
        ServerBattleState{slot: slot, ships: ships}
    }
    
    pub fn run(&mut self) {
        loop {
            match self.slot.receive() {
                Joined(client_id) => {
                    println!("Client {} joined battle {}", client_id, self.slot.id());
                    let mut packet = OutPacket::new();
                    packet.write_i32(42).unwrap();
                    packet.write_u32(444422).unwrap();
                    packet.write_i32(64).unwrap();
                    self.slot.send(client_id, packet);
                },
                ReceivedPacket(client_id, packet) => {
                    println!("Battle {} received packet from {} of length {}", self.slot.id(), client_id, packet.len());
                },
                _ => {}
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Client

pub struct ClientBattleState {
    client: Client,
}