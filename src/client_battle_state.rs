use std::collections::HashMap;
use std::io::Timer;
use std::time::Duration;

use rsfml::graphics::{RenderTarget, RenderWindow, Color};

use battle_state_packets::{ClientPacketId, Plan};
use input::InputSystem;
use net::{Client, ClientId, InPacket, OutPacket};
use render::Renderer;
use sfml_renderer::SfmlRenderer;
use ship::Ship;
use sim_element::SimElement;

pub struct ClientBattleState {
    client: Client,
    
    // All the ships involved in this battle
    ships: HashMap<ClientId, Ship>,
}

impl ClientBattleState {
    pub fn new(client: Client) -> ClientBattleState {
        ClientBattleState{client: client, ships: HashMap::new()}
    }
    
    pub fn run(&mut self, renderer: &mut SfmlRenderer, input: &mut InputSystem) {
        renderer.create_render_target(0, 0, 500, 500);
        renderer.create_render_target(512, 0, 500, 500);
    
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
            // Do planning
            let plan_end_signal = timer.oneshot(Duration::seconds(10));
            while renderer.window.is_open() {
                match plan_end_signal.try_recv() {
                    Ok(_) => break, // Received timeup signal
                    Err(_) => {}
                }
                
                // Update input
                input.update(&mut renderer.window);
                
                // Do planning stuff
                self.plan();
                
                // Render
                (&mut renderer.window as &mut RenderTarget).clear(&Color::black());
                self.draw(renderer, false, 0f32);
                renderer.display();
            }
        
            // Send plans
            let packet = self.build_plans_packet();
            self.client.send(&packet);
            
            // Wait for simulation results
            self.receive_simulation_results();
            
            // Simulate
            /*let simulate_end_signal = timer.oneshot(Duration::seconds(5));
            while renderer.window.is_open() {
                match simulate_end_signal.try_recv() {
                    Ok(_) => break, // Received timeup signal
                    Err(_) => {}
                }
                
                // Update input
                input.update(&mut renderer.window);
                
                // Do planning stuff
                self.simulate(0);
                
                // Render
                (&mut renderer.window as &mut RenderTarget).clear(&Color::black());
                self.draw(renderer, true, 0f32);
                renderer.display();
            }*/
        }
    }
    
    fn plan(&mut self) {
    }
    
    fn build_plans_packet(&mut self) -> OutPacket {
        let mut packet = OutPacket::new();
        packet.write_u8(Plan as u8).unwrap();
        
        self.apply_to_sim_elements(|sim_element| {
            sim_element.write_plans(&mut packet);
        });

        packet
    }
    
    fn receive_simulation_results(&mut self) {
        let mut packet = self.client.receive();
        let id = match (packet.read_u8()) {
            Ok(id) => id,
            Err(e) => fail!("Failed to read simulation results packet ID: {}", e)
        };
        
        self.apply_to_sim_elements(|sim_element| {
            sim_element.read_results(&mut packet);
        });
    }
    
    fn simulate(&mut self, time: u32) {
        self.apply_to_sim_elements(|sim_element| {
            sim_element.on_simulation_time(&self.ships, time);
        });
    }
    
    fn draw(&self, renderer: &mut Renderer, simulating: bool, time: f32) {
        for ship in self.ships.values() {
            for module in ship.modules.iter() {
                module.borrow().draw(renderer, simulating, time);
            }
        }
    }
    
    fn apply_to_sim_elements(&self, f: |&mut SimElement|) {
        for (_, ship) in self.ships.iter() {
            for module in ship.modules.iter() {
                f(module.borrow_mut().deref_mut() as &mut SimElement);
            }
        }
    }
}