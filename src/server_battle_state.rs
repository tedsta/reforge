use std::collections::HashMap;

use battle_state_packets::{Plan, ServerPacketId};
use net::{ClientId, ServerSlot, Joined, ReceivedPacket, InPacket, OutPacket};
use ship::Ship;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server

pub struct ServerBattleState {
    slot: ServerSlot,
    ships: HashMap<ClientId, Ship>,
    turn_number: uint,
}

impl ServerBattleState {
    pub fn new(slot: ServerSlot, ships: HashMap<ClientId, Ship>) -> ServerBattleState {
        ServerBattleState {
            slot: slot,
            ships: ships,
            turn_number: 0,
        }
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
                ReceivedPacket(client_id, mut packet) => { self.handle_packet(client_id, &mut packet); },
                _ => {}
            }
        }
    }
    
    pub fn handle_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
        println!("Battle {} received packet from {} of length {}", self.slot.id(), client_id, packet.len());
    
        let id: ServerPacketId = match packet.read_u8() {
            Ok(id) => match FromPrimitive::from_u8(id) {
                Some(id) => id,
                None => {
                    println!("Received packet with invalid ID from client {}", client_id);
                    return;
                }
            },
            Err(e) => {
                println!("Received empty packet from client {}: {}", client_id, e);
                return;
            }
        };
        
        match id {
            Plan => {
                println!("Yay plans processing on server!");
            },
        }
    }
}