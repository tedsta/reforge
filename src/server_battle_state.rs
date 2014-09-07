use std::collections::HashMap;

use battle_state_packets::{Plan, ServerPacketId};
use net::{ClientId, ServerSlot, Joined, ReceivedPacket, InPacket, OutPacket};
use ship::Ship;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server

pub struct ServerBattleState {
    slot: ServerSlot,
    ships: HashMap<ClientId, Ship>,
    
    received_plans: Vec<ClientId>,
    turn_number: uint,
}

impl ServerBattleState {
    pub fn new(slot: ServerSlot, ships: HashMap<ClientId, Ship>) -> ServerBattleState {
        ServerBattleState {
            slot: slot,
            ships: ships,
            received_plans: vec!(),
            turn_number: 0,
        }
    }
    
    pub fn run(&mut self) {
        loop {
            match self.slot.receive() {
                Joined(client_id) => {
                    println!("Client {} joined battle {}", client_id, self.slot.id());
                    
                    // Send the player all the ships
                    let mut packet = OutPacket::new();
                    packet.write_u32(self.ships.len() as u32).unwrap(); // The number of ships in the packet
                    for (ship_client_id, ship) in self.ships.iter() {
                        packet.write_u32(*ship_client_id).unwrap();
                        packet.write(ship).unwrap();
                    }
                    self.slot.send(client_id, packet);
                },
                ReceivedPacket(client_id, mut packet) => { self.handle_packet(client_id, &mut packet); },
                _ => {}
            }
        }
    }
    
    fn handle_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
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
            
                self.received_plans.push(client_id);
                if self.received_plans.len() == self.ships.len() {
                    println!("Got all the plans!");
                    self.received_plans.clear();
                }
            },
        }
    }
}