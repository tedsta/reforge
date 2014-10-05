use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Timer;
use std::time::Duration;

use time;

use rsfml::graphics::{RenderWindow, RenderTarget, Color};

use battle_state::{ClientPacketId, Plan, TICKS_PER_SECOND};
use input::InputSystem;
use net::{Client, ClientId, InPacket, OutPacket};
use render;
use render::{Renderer};
use sfml_renderer::SfmlRenderer;
use ship::{Ship, ShipRef};
use sim_element::SimElement;
use vec::{Vec2, Vec2f};

pub struct ShipRenderArea {
    render_target: render::RenderTarget,
    position: Vec2f,
    ship_id: ClientId,
}

pub struct ClientBattleState {
    client: Client,
    
    // All the ships involved in this battle
    ships: HashMap<ClientId, ShipRef>,
    
    // The ships' render areas
    render_areas: Vec<ShipRenderArea>,
}

impl ClientBattleState {
    pub fn new(client: Client) -> ClientBattleState {
        ClientBattleState{client: client, ships: HashMap::new(), render_areas: vec!()}
    }
    
    pub fn run(&mut self, renderer: &mut SfmlRenderer, input: &mut InputSystem) {    
        // Receive all of the ships participating in this battle
        let mut packet = self.client.receive();
        let ship_count = packet.read_u32().unwrap();
        for i in range(0, ship_count) {
            let client_id = packet.read_u32().unwrap();
            let mut ship: ShipRef = packet.read().unwrap();
            let render_target = renderer.create_render_target(500, 500);
            ship.borrow_mut().render_target = render_target;
            self.render_areas.push(ShipRenderArea{render_target: render_target, position: Vec2{x: (i*512) as f32, y: 0f32}, ship_id: client_id});
            self.ships.insert(client_id, ship);
        }
    
        loop {
            // Do planning
            let start_time = time::now().to_timespec();
            while renderer.window.is_open() {
                let current_time = time::now().to_timespec();
                let elapsed_time = current_time - start_time;
                if elapsed_time.num_seconds() >= 10 {
                    break;
                }
                
                // Update input
                input.update(&mut renderer.window);
                
                // Do planning stuff
                self.plan();
                
                // Render
                (&mut renderer.window as &mut RenderTarget).clear(&Color::black());
                renderer.clear_render_targets();
                
                self.draw(renderer, true, 0f32);
                renderer.display_render_targets();
                
                for render_area in self.render_areas.iter() {
                    renderer.draw_texture_vec(render_area.render_target.texture, &render_area.position);
                }
                
                renderer.window.display();
            }
        
            // Send plans
            let packet = self.build_plans_packet();
            self.client.send(&packet);
            
            // Wait for simulation results
            self.receive_simulation_results();
            
            ////////////////////////////////
            // Simulate
            
            // Before simulation
            self.apply_to_sim_elements(|sim_element| {
                sim_element.before_simulation(&self.ships);
            });
            
            // Simulation
            let start_time = time::now().to_timespec();
            let mut last_time = time::now().to_timespec();
            let mut next_tick = 0;
            while renderer.window.is_open() {
                // Cap the framerate
                while (time::now().to_timespec()-start_time).num_milliseconds() < 1 {}
            
                // Get current time
                let current_time = time::now().to_timespec();
                
                // Calculate total elapsed time
                let elapsed_time = current_time - start_time;
                
                // Check if we're done
                if elapsed_time.num_seconds() >= 5 {
                    break;
                }
                
                // 20 ticks per second
                let tick = (elapsed_time.num_milliseconds() as u32)/(1000/TICKS_PER_SECOND);
                
                // Calculate elapsed time in seconds as f32
                let elapsed_seconds = (elapsed_time.num_milliseconds() as f32)/1000f32;
                
                // Prepare last_time for next frame
                last_time = current_time;
                
                // Update input
                input.update(&mut renderer.window);
                
                // Simulate any new ticks
                for t in range(next_tick, next_tick + tick-next_tick+1) {
                    self.simulate(t);
                }
                next_tick = tick+1;
                
                // Render
                (&mut renderer.window as &mut RenderTarget).clear(&Color::black());
                renderer.clear_render_targets();
                
                self.draw(renderer, true, elapsed_seconds);
                renderer.display_render_targets();
                
                for render_area in self.render_areas.iter() {
                    renderer.draw_texture_vec(render_area.render_target.texture, &render_area.position);
                }
                
                renderer.window.display();
            }
            
            // After simulation
            self.apply_to_sim_elements(|sim_element| {
                sim_element.after_simulation(&self.ships);
            });
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
            for module in ship.borrow().modules.iter() {
                module.borrow_mut().draw(renderer, simulating, time);
            }
        }
    }
    
    fn apply_to_sim_elements(&self, f: |&mut SimElement|) {
        for (_, ship) in self.ships.iter() {
            for module in ship.borrow().modules.iter() {
                f(module.borrow_mut().deref_mut() as &mut SimElement);
            }
        }
    }
}