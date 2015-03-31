use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{Sender, Receiver};
use time;

use ai::run_ai;
use battle_state::{BattleContext, ClientPacketId, ServerPacketId};
use login::AccountBox;
use module::Module;
use net::{ClientId, ServerSlot, ServerSlotId, SlotInMsg, InPacket, OutPacket};
use ship::{Ship, ShipId, ShipRef, ShipStored};
use sim::SimEvents;

pub struct SectorState {
    slot: ServerSlot,
    star_map_slot_id: ServerSlotId,
    
    // All the clients' accounts
    accounts: HashMap<ClientId, AccountBox>,
}

impl SectorState {
    pub fn new(slot: ServerSlot, star_map_slot_id: ServerSlotId) -> SectorState {
        SectorState {
            slot: slot,
            star_map_slot_id: star_map_slot_id,
            accounts: HashMap::new(),
        }
    }
    
    pub fn run(&mut self, to_map_sender: Sender<AccountBox>, from_map_receiver: Receiver<AccountBox>) {
        loop {        
            ///////////////////////////////////////////////////////////
            // Receiver ServerSlot messages
            if let Ok(msg) = self.slot.try_receive() {
                match msg {
                    SlotInMsg::Joined(client_id) => {
                        println!("Client {} joined battle {}", client_id, self.slot.get_id());
                    },
                    SlotInMsg::ReceivedPacket(client_id, mut packet) => { self.handle_packet(client_id, &mut packet); },
                    _ => {}
                }
            }
            
            ///////////////////////////////////////////////////////////
            // Receive new clients
            if let Ok(mut account) = from_map_receiver.try_recv() {
                let client_id = account.client_id.expect("This must have a client ID");
                
                // Add the client to the waiting list
                self.clients_waiting.insert(client_id);
                
                // Get the ship out of storage
                let ship_stored = account.ship.take().expect("This account must have a ship");
                let ship = ship_stored.to_ship(Some(client_id));
                
                // Add the player's account
                self.accounts.insert(client_id, account);
                
                // Send initial join packet
                let mut packet = OutPacket::new();
                packet.write(&ship);
                packet.write(&self.sent_new_ships); // Whether or not to start at simulation instead of planning phase
                packet.write(&self.context.ships_list).unwrap();
                self.slot.send(client_id, packet);
                
                // Add the player's ship
                let ship = Rc::new(RefCell::new(ship));
                self.context.add_ship(ship.clone());
                self.ships_to_add.push(ship);
            }
        }
    }
    
    fn handle_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
        let id: ServerPacketId = match packet.read() {
            Ok(id) => id,
            Err(e) => {
                println!("Received invalid packet from client {}: {}", client_id, e);
                return;
            }
        };
        
        match id {
            ServerPacketId::Plan => {
                self.received_plans.insert(client_id);
                
                // Handle the plans
                self.handle_plans_packet(client_id, packet);
                
                println!("Received plans packet from {} for turn {}", client_id, self.turn_number);
 
                if self.received_plans == self.clients_active {
                    // TODO decide if we should simulate turn a little early here.
                }
            },
        }
    }
}
