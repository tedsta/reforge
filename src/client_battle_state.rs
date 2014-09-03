use std::collections::HashMap;
use std::io::Timer;
use std::time::Duration;

use rsfml::graphics::{RenderWindow, Color};

use battle_state_packets::{Plan};
use input::InputSystem;
use net::{Client, InPacket, OutPacket};
use ship::Ship;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Client

pub struct ClientBattleState {
    client: Client,
}

impl ClientBattleState {
    pub fn new(client: Client) -> ClientBattleState {
        ClientBattleState{client: client}
    }
    
    pub fn run(&mut self, window: &mut RenderWindow, input: &mut InputSystem) {
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
    
    pub fn plan(&mut self) {
    }
}