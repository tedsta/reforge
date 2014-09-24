use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, TreeMap};

use battle_state_packets::{Plan, ServerPacketId, SimResults};
use module::Module;
use net::{ClientId, ServerSlot, Joined, ReceivedPacket, InPacket, OutPacket};
use ship::Ship;
use sim_element::SimElement;

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
                
                // Handle the plans
                self.handle_plans_packet(packet);
                
                if self.received_plans.len() == self.ships.len() {
                    println!("Got all the plans!");
                    
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
        self.apply_to_sim_elements_with_ships(|sim_element, ships| {
            sim_element.before_simulation(ships);
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
        self.apply_to_sim_elements_with_ships(|sim_element, ships| {
            sim_element.after_simulation(ships);
        });
    }
    
    fn apply_to_sim_elements(&mut self, f: |&mut SimElement|) {
        for (_, ship) in self.ships.mut_iter() {
            for module in ship.modules.mut_iter() {
                f(module as &mut SimElement);
            }
        }
    }
    
    fn apply_to_sim_elements_with_ships(&mut self, f: |&mut SimElement, &mut HashMap<ClientId, Ship>|) {
        for (_, ship) in self.ships.mut_iter() {
            for module in ship.modules.mut_iter() {
                f(module as &mut SimElement, &mut self.ships);
            }
        }
    }
    
    fn simulate(&mut self) {
        let mut time_slots: TreeMap<u32, Vec<Rc<RefCell<&mut Module>>>> = TreeMap::new();
        
        for (_, ship) in self.ships.mut_iter() {
            for module in ship.modules.mut_iter() {
                let times = module.get_critical_times();
                let module_ref = Rc::new(RefCell::new(module));
                
                for time in times.iter() {                
                    if time_slots.contains_key(time) {
                        time_slots.find_mut(time).unwrap().push(module_ref.clone())
                    } else {
                        time_slots.insert(*time, vec![module_ref.clone()]);
                    }
                }
            }
        }
        
        for (time, sim_element_refs) in time_slots.mut_iter() {
            for sim_element_ref in sim_element_refs.mut_iter() {
                sim_element_ref.borrow_mut().on_simulation_time(&mut self.ships, *time);
            }
        }
    }
}