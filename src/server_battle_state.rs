use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, TreeMap};

use battle_state::{Plan, ServerPacketId, SimResults};
use module::Module;
use net::{ClientId, ServerSlot, Joined, ReceivedPacket, InPacket, OutPacket};
use ship::ShipRef;
use sim_element::SimElement;

pub struct ServerBattleState {
    slot: ServerSlot,
    ships: HashMap<ClientId, ShipRef>,
    
    received_plans: Vec<ClientId>,
    turn_number: u32,
}

impl ServerBattleState {
    pub fn new(slot: ServerSlot, ships: HashMap<ClientId, ShipRef>) -> ServerBattleState {
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
                self.received_plans.push(client_id);
                
                // Handle the plans
                self.handle_plans_packet(packet);
                
                if self.received_plans.len() == self.ships.len() {
                    
                    self.do_simulation();
                    
                    // Reset everything for the next turn
                    self.received_plans.clear();
                    self.turn_number += 1;
                }
            },
        }
    }
    
    fn handle_plans_packet(&mut self, packet: &mut InPacket) {
        self.apply_to_sim_elements(|sim_element| {
            sim_element.read_plans(packet);
        });
    }
    
    fn do_simulation(&mut self) {
        // Pre simulation
        self.apply_to_sim_elements(|sim_element| {
            sim_element.before_simulation(&self.ships);
        });
    
        // Write results packet
        let mut packet = OutPacket::new();
        packet.write_u8(SimResults as u8).unwrap();
        
        self.apply_to_sim_elements(|sim_element| {
            sim_element.write_results(&mut packet);
        });
        
        self.slot.broadcast(packet);
        
        // Simulation!!!
        self.simulate();
        
        // Post simulation
        self.apply_to_sim_elements(|sim_element| {
            sim_element.after_simulation(&self.ships);
        });
    }
    
    fn apply_to_sim_elements(&self, f: |&mut SimElement|) {
        for (_, ship) in self.ships.iter() {
            for module in ship.borrow().modules.iter() {
                f(module.borrow_mut().deref_mut() as &mut SimElement);
            }
        }
    }
    
    fn simulate(&mut self) {
        for i in range(0, 100) {
            self.apply_to_sim_elements(|sim_element| {
                sim_element.on_simulation_time(&self.ships, i);
            });
        }
    
        /*let mut time_slots: TreeMap<u32, Vec<Rc<RefCell<Module>>>> = TreeMap::new();
        
        for (_, ship) in self.ships.iter() {
            for module in ship.modules.iter() {
                let times = module.borrow().get_critical_times();
                let module_ref = module.clone();
                
                for time in times.iter() {                
                    if time_slots.contains_key(time) {
                        time_slots.find_mut(time).unwrap().push(module_ref.clone())
                    } else {
                        time_slots.insert(*time, vec![module_ref.clone()]);
                    }
                }
            }
        }
        
        for (time, sim_elements) in time_slots.iter() {
            for sim_elements in sim_elements.iter() {
                sim_elements.borrow_mut().on_simulation_time(&mut self.ships, *time);
            }
        }*/
    }
}