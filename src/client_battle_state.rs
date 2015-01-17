use time;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use piston::event::Events;
use opengl_graphics::Gl;
use sdl2_window::Sdl2Window;

use asset_store::AssetStore;
use battle_state::{BattleContext, ClientPacketId, ServerPacketId, TICKS_PER_SECOND};
use net::{Client, OutPacket};
use ship::{Ship, ShipId, ShipRef};
use sim::{SimEvents, SimVisuals};
use space_gui::SpaceGui;

pub struct ClientBattleState {
    client: Client,
    
    // Context holding all the things involved in this battle
    context: BattleContext,
    
    // The player's ship
    player_ship: ShipRef,
    
    ships_to_add: Vec<Ship>,
    ships_to_remove: Vec<ShipId>,
}

impl ClientBattleState {
    pub fn new(client: Client, context: BattleContext) -> ClientBattleState {
        let player_ship = context.get_ship_by_client_id(client.get_id()).clone();
        ClientBattleState {
            client: client,
            context: context,
            player_ship: player_ship,
            ships_to_add: vec!(),
            ships_to_remove: vec!(),
        }
    }
    
    pub fn run(&mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, asset_store: &AssetStore) {
        let mut gui = SpaceGui::new(asset_store, &self.context, self.client.get_id());
    
        let mut sim_visuals = SimVisuals::new();
    
        loop {
            ////////////////////////////////
            // Plan
            
            // Add planning visuals
            sim_visuals.clear();
            self.context.add_plan_visuals(asset_store, &mut sim_visuals);
            
            // Store mouse coordinates
            let (mut mouse_x, mut mouse_y) = (0f64, 0f64);
            
            // Record start time
            let start_time = time::now().to_timespec();
            let mut last_time = 0.0;
            
            // Run planning loop
            for e in Events::new(window) {
                use piston::event;
                use piston::input;
                use piston::event::*;

                let e: event::Event<input::Input> = e;
            
                // Keep track of time, break when planning is done
                let current_time = time::now().to_timespec();
                let elapsed_time = current_time - start_time;
                if elapsed_time.num_seconds() >= 5 {
                    break;
                }
                
                // Calculate elapsed time in seconds as f64
                let elapsed_seconds = (elapsed_time.num_milliseconds() as f64)/1000f64;
                let dt = elapsed_seconds - last_time;
                last_time = elapsed_seconds;
            
                // Forward events to GUI
                gui.event(&e, self.player_ship.borrow_mut().deref_mut());
                
                // Render GUI
                e.render(|&mut: args: &RenderArgs| {
                    gl.draw([0, 0, args.width as i32, args.height as i32], |: c, gl| {
                        gui.draw_planning(&c, gl, asset_store, &mut sim_visuals, self.player_ship.borrow().deref(), elapsed_seconds, dt);
                    });
                });
            }
            
            self.player_ship.borrow_mut().apply_module_plans();
        
            // Send plans
            let packet = self.build_plans_packet();
            self.client.send(&packet);
            
            // Wait for simulation results
            self.receive_simulation_results();
            
            ////////////////////////////////
            // Simulate
            
            let mut sim_events = SimEvents::new();
            
            // Before simulation
            sim_visuals.clear();
            self.context.before_simulation(&mut sim_events);
            self.context.add_simulation_visuals(asset_store, &mut sim_visuals);
            
            // Simulation
            let start_time = time::now().to_timespec();
            let mut last_time = 0.0;
            let mut next_tick = 0;
            for e in Events::new(window) {
                use piston::event;
                use piston::input;
                use piston::event::*;

                let e: event::Event<input::Input> = e;
            
                // Keep track of time, break when simulation is done
                let current_time = time::now().to_timespec();
                let elapsed_time = current_time - start_time;
                if elapsed_time.num_seconds() >= 5 {
                    break;
                }
                
                // Calculate current tick
                let tick = (elapsed_time.num_milliseconds() as u32)/(1000/TICKS_PER_SECOND);
                
                // Calculate elapsed time in seconds as f64
                let elapsed_seconds = (elapsed_time.num_milliseconds() as f64)/1000f64;
                let dt = elapsed_seconds - last_time;
                last_time = elapsed_seconds;
                
                // Simulate any new ticks
                for t in range(next_tick, next_tick + tick-next_tick+1) {
                    sim_events.apply_tick(t);
                }
                next_tick = tick+1;
            
                // Forward events to GUI
                gui.event(&e, self.player_ship.borrow_mut().deref_mut());
                
                // Render GUI
                e.render(|&mut: args: &RenderArgs| {
                    gl.draw([0, 0, args.width as i32, args.height as i32], |: c, gl| {
                        gui.draw_simulating(&c, gl, asset_store, &mut sim_visuals, self.player_ship.borrow().deref(), elapsed_seconds, dt);
                    });
                });
            }
            
            // After simulation
            self.context.after_simulation();
            
            // Handle ships to add and remove
            for ship in self.ships_to_remove.drain() {
                gui.remove_lock(ship);
            
                self.context.remove_ship(ship);
            }
        
            for ship in self.ships_to_add.drain() {
                let ship = Rc::new(RefCell::new(ship));
                
                if ship.borrow().client_id == Some(self.client.get_id()) {
                    self.player_ship = ship.clone();
                } else {
                    gui.try_lock(&ship);
                }
                self.context.add_ship(ship);
            }
        }
    }
    
    fn build_plans_packet(&mut self) -> OutPacket {
        let mut packet = OutPacket::new();
        match packet.write(&ServerPacketId::Plan) {
            Ok(()) => {},
            Err(_) => panic!("Failed to write plan packet ID"),
        }
        
        self.player_ship.borrow().write_plans(&mut packet);

        packet
    }
    
    fn receive_simulation_results(&mut self) {
        let mut packet = self.client.receive();
        match (packet.read::<ClientPacketId>()) {
            Ok(ref id) if *id != ClientPacketId::SimResults => panic!("Expected SimResults, got something else"),
            Err(e) => panic!("Failed to read simulation results packet ID: {}", e),
            _ => {}, // All good!
        };
        
        // Results packet has both plans and results
        self.context.read_plans(&mut packet);
        self.context.read_results(&mut packet);
        
        self.ships_to_add = packet.read().ok().expect("Failed to read ships to add from results packet");
        self.ships_to_remove = packet.read().ok().expect("Failed to read ships to remove from results packet");
    }
}
