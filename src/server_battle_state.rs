use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, TreeMap};

use battle_state::{BattleContext, Plan, ServerPacketId, SimResults};
use module::Module;
use net::{ClientId, ServerSlot, Joined, ReceivedPacket, InPacket, OutPacket};
use ship::Ship;
use sim_element::SimElement;

pub struct ServerBattleState {
    slot: ServerSlot,
    
    // Context holding all the things involved in this battle
    context: BattleContext,
    
    received_plans: Vec<ClientId>,
    turn_number: u32,
}

impl ServerBattleState {
    pub fn new(slot: ServerSlot, context: BattleContext) -> ServerBattleState {
        ServerBattleState {
            slot: slot,
            context: context,
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
                    packet.write(&self.context);
                    self.slot.send(client_id, packet);
                },
                ReceivedPacket(client_id, mut packet) => { self.handle_packet(client_id, &mut packet); },
                _ => {}
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
            Plan => {
                self.received_plans.push(client_id);
                
                // Handle the plans
                self.handle_plans_packet(packet);
 
                if self.received_plans.len() == self.context.get_num_ships() {
                    
                    self.do_simulation();
                    
                    // Reset everything for the next turn
                    self.received_plans.clear();
                    self.turn_number += 1;
                }
            },
        }
    }
    
    fn handle_plans_packet(&mut self, packet: &mut InPacket) {
        self.context.apply_to_sim_elements(|sim_element| {
            sim_element.read_plans(packet);
        });
    }
    
    fn do_simulation(&mut self) {
        // Pre simulation
        self.context.apply_to_sim_elements(|sim_element| {
            sim_element.before_simulation(&self.context);
        });

        // Write results packet
        let mut packet = OutPacket::new();
        packet.write(&SimResults).unwrap();
        
        self.context.apply_to_sim_elements(|sim_element| {
            sim_element.write_results(&mut packet);
        });
        
        self.slot.broadcast(packet);
        
        // Simulation!!!
        self.simulate();
        
        // Post simulation
        self.context.apply_to_sim_elements(|sim_element| {
            sim_element.after_simulation(&self.context);
        });
    }
    
    fn simulate(&mut self) {
        for i in range(0, 100) {
            self.context.apply_to_sim_elements(|sim_element| {
                sim_element.on_simulation_time(&self.context, i);
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