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

pub struct ClientBattleState<'r> {
    client: Client,
    
    // All the ships involved in this battle
    ships: HashMap<ClientId, Ship>,
    
    sim_elements: Vec<&'r mut SimElement + 'static>,
}

impl<'r> ClientBattleState<'r> {
    pub fn new(client: Client) -> ClientBattleState<'r> {
        ClientBattleState{client: client, ships: HashMap::new(), sim_elements: vec!()}
    }
    
    pub fn run(&mut self, window: &mut RenderWindow, input: &mut InputSystem) {
        // Receive all of the ships participating in this battle
        let mut packet = self.client.receive();
        let ship_count = packet.read_u32().unwrap();
        for _ in range(0, ship_count) {
            let client_id = packet.read_u32().unwrap();
            let ship: Ship = packet.read().unwrap();
            for module in ship.modules.mut_iter() {
                self.sim_elements.push(&mut **module as &mut SimElement);
            }
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
            let mut packet = OutPacket::new();
            packet.write_u8(Plan as u8).unwrap();
            self.client.send(&packet);
            
            // Wait for simulation results
            
            // Simulate
        }
    }
    
    fn plan(&mut self) {
    }
    
    fn handle_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
        let id: ClientPacketId = match packet.read_u8() {
            Ok(id) => match FromPrimitive::from_u8(id) {
                Some(id) => id,
                None => {
                    println!("Received packet with invalid ID from client {}", client_id);
                    return;
                }
            },
            Err(e) => {
                println!("Failed to read packet ID from packet: {}", e);
                return;
            }
        };
        
        match id {
            _ => {}
        }
    }
}