use std::collections::HashMap;
use std::io::Timer;
use std::time::Duration;

use rsfml::graphics::{RenderWindow, Color};

use battle_state_packets::{ClientPacketId, Plan};
use input::InputSystem;
use net::{Client, ClientId, InPacket, OutPacket};
use ship::Ship;
use sim_element::SimElement;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Client

pub struct ClientBattleState {
    client: Client,
    
    // All the ships involved in this battle
    ships: HashMap<ClientId, Ship>,
}

impl ClientBattleState {
    pub fn new(client: Client) -> ClientBattleState {
        ClientBattleState{client: client, ships: HashMap::new()}
    }
    
    pub fn run(&mut self, window: &mut RenderWindow, input: &mut InputSystem) {
        // Receive all of the ships participating in this battle
        let mut packet = self.client.receive();
        let ship_count = packet.read_u32().unwrap();
        for _ in range(0, ship_count) {
            let client_id = packet.read_u32().unwrap();
            let ship = packet.read().unwrap();
            self.ships.insert(client_id, ship);
        }
    
        let mut timer = Timer::new().unwrap();
        loop {
            let plan_time_signal = timer.oneshot(Duration::seconds(10));
        
            // Do planning
            while window.is_open() {
                match plan_time_signal.try_recv() {
                    Ok(_) => break, // Received timeup signal
                    Err(_) => {}
                }
                
                // Update input
                input.update(window);
                
                // Do planning stuff
                self.plan();
                
                // Render
                window.clear(&Color::black());
                window.display();
            }
        
            // Send plans
            let packet = self.build_plans_packet();
            self.client.send(&packet);
            
            // Wait for simulation results
            self.wait_for_simulation_results();
            
            // Simulate
        }
    }
    
    fn plan(&mut self) {
    }
    
    fn build_plans_packet(&mut self) -> OutPacket {
        let mut packet = OutPacket::new();
        packet.write_u8(Plan as u8).unwrap();
        
        let sim_elements = self.build_sim_elements_vec();
        
        for sim_element in sim_elements.iter() {
            sim_element.write_plans(&mut packet);
        }

        packet
    }
    
    fn build_sim_elements_vec(&mut self) -> Vec<&mut SimElement> {
        let mut elements = vec!();
        
        for (_, ship) in self.ships.mut_iter() {
            for module in ship.modules.mut_iter() {
                elements.push(module as &mut SimElement);
            }
        }
        
        elements
    }
    
    fn wait_for_simulation_results(&mut self) {
        let mut packet = self.client.receive();
        
        let count = packet.read_u32();
    }
}