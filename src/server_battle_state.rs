use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, TreeMap};

use battle_state::{BattleContext, ClientPacketId, ServerPacketId};
use module::Module;
use net::{ClientId, ServerSlot, SlotInMsg, InPacket, OutPacket};
use ship::Ship;
use sim::SimEvents;

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
                SlotInMsg::Joined(client_id) => {
                    println!("Client {} joined battle {}", client_id, self.slot.get_id());
                    
                    // Send the player all the ships
                    let mut packet = OutPacket::new();
                    packet.write(&self.context.ships_client_id);
                    self.slot.send(client_id, packet);
                },
                SlotInMsg::ReceivedPacket(client_id, mut packet) => { self.handle_packet(client_id, &mut packet); },
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
            ServerPacketId::Plan => {
                self.received_plans.push(client_id);
                
                // Handle the plans
                self.handle_plans_packet(client_id, packet);
 
                if self.received_plans.len() == self.context.get_num_ships() {
                    // Do server-side precalculations
                    self.context.server_preprocess();
                    
                    // Build the results packet
                    let results_packet = self.build_results_packet();
                    self.slot.broadcast(results_packet);
                    
                    // Run the simulation
                    self.do_simulation();
                    
                    // Reset everything for the next turn
                    self.received_plans.clear();
                    self.turn_number += 1;
                }
            },
        }
    }
    
    fn handle_plans_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
        self.context.get_ship_by_client_id(client_id).borrow_mut().read_plans(&self.context, packet);
    }
    
    fn do_simulation(&mut self) {
        let mut sim_events = SimEvents::new();
    
        // Pre simulation
        self.context.before_simulation(&mut sim_events);
        
        // Simulation!!!
        self.simulate(&mut sim_events);
        
        // Post simulation
        self.context.after_simulation();
    }
    
    fn simulate(&mut self, sim_events: &mut SimEvents) {
        for tick in range(0u32, 100) {
            sim_events.apply_tick(tick);
        }
    }
    
    fn build_results_packet(&mut self) -> OutPacket {
        let mut packet = OutPacket::new();
        match packet.write(&ClientPacketId::SimResults) {
            Ok(()) => {},
            Err(e) => panic!("Failed to write results packet ID: {}", e),
        }
        
        // The results packet has both the plans and the results, because clients need both
        self.context.write_plans(&mut packet);
        self.context.write_results(&mut packet);

        packet
    }
}