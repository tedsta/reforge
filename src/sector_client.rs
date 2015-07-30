use std::cell::RefCell;
use std::cmp;
use std::io;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use time;

use piston::event::Events;
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use glutin_window::GlutinWindow;
use piston::window::Window;

use asset_store::AssetStore;
use battle_context::{BattleContext, TICKS_PER_SECOND};
use chat::ChatGui;
use module::ModelStore;
use net::{Client, InPacket, OutPacket};
use packet_types::{ClientBattlePacket, ServerBattlePacket};
use sector_data::SectorData;
use ship::{Ship, ShipId, ShipIndex};
use sim::{SimEvents, SimEffects};
use space_gui::{SpaceGui, SpaceGuiAction};

pub struct ClientBattleState<'a> {
    client: &'a mut Client,
    
    // Context holding all the things involved in this battle
    bc: BattleContext,
    
    // The player's ship
    player_ship: ShipIndex,
    
    new_ships_pre: Option<InPacket>,
    results: Option<InPacket>,
    new_ships_post: Option<InPacket>,
    
    final_ticks: Option<u8>,
}

impl<'a> ClientBattleState<'a> {
    pub fn new(client: &'a mut Client, bc: BattleContext) -> ClientBattleState<'a> {
        let player_ship = bc.get_ship_by_client_id(client.get_id()).index;
        ClientBattleState {
            client: client,
            bc: bc,
            player_ship: player_ship,
            new_ships_pre: None,
            results: None,
            new_ships_post: None,
            final_ticks: None,
        }
    }
    
    pub fn run(&mut self,
               window: &Rc<RefCell<GlutinWindow>>,
               gl: &mut GlGraphics,
               glyph_cache: &mut GlyphCache,
               asset_store: &AssetStore,
               model_store: &ModelStore,
               chat_gui: &mut ChatGui,
               sectors: Vec<SectorData>,
               server_results_sent: bool) {
        use piston::window::Window;
    
        let ref mut gui = SpaceGui::new(asset_store, &self.bc, chat_gui, sectors, self.player_ship);
    
        let ref mut sim_effects = SimEffects::new();
        
        // TODO display joining screen here
        
        if server_results_sent {
            // Wait for the tick
            loop {
                // We might get chat packets here
                let tick_packet = self.client.receive();
                let ticked = self.handle_packet(gui, tick_packet);
                
                if ticked {
                    break;
                }
            }
        }
        
        // Get first turn's results
        loop {
            // Loop until tick packet is received
            let packet = self.client.receive();
            let ticked = self.handle_packet(gui, packet);
            if ticked {
                break;
            }
        }

        loop {
            ////////////////////////////////
            // Simulate
            
            if let Some(ref mut ticks_left) = self.final_ticks {
                if *ticks_left == 0 {
                    break;
                } else {
                    *ticks_left -= 1;
                }
            }
            
            // Receive simulation results
            let mut new_ships_pre = self.new_ships_pre.take().expect("New ships pre packet must exist here");
            let mut results = self.results.take().expect("Results packet must exist here");
            let mut new_ships_post = self.new_ships_post.take().expect("New ships post packet must exist here");
            
            self.handle_new_ships_packet(gui, &mut new_ships_pre);
            self.handle_simulation_results(&mut results);
            
            self.run_simulation_phase(window, gl, glyph_cache, asset_store, model_store, gui, sim_effects);
            
            // Receive ships after sim
            self.handle_new_ships_packet(gui, &mut new_ships_post);
            
            // Check if it's time to exit
            if window.borrow().should_close() { break; }
        }
    }
    
    fn run_simulation_phase(&mut self,
                            window: &Rc<RefCell<GlutinWindow>>,
                            gl: &mut GlGraphics,
                            glyph_cache: &mut GlyphCache,
                            asset_store: &AssetStore,
                            model_store: &ModelStore,
                            gui: &mut SpaceGui,
                            mut sim_effects: &mut SimEffects) -> bool {
        // Unlock any exploding or jumping ships
        let ships_to_unlock: Vec<ShipIndex> =
            self.bc.ships_iter()
                .filter_map(|s| if s.jumping || s.exploding { Some(s.index) } else { None })
                .collect();
        
        for ship_index in ships_to_unlock {
            self.bc.on_ship_removed(ship_index);
            gui.plans.on_ship_removed(ship_index);
        }
        
        let mut logging_out = false;
        
        let mut sim_events = SimEvents::new();
            
        // Before simulation
        sim_effects.reset();
        self.bc.before_simulation(model_store, &mut sim_events);
        self.bc.add_simulation_effects(asset_store, model_store, &mut sim_effects);
        
        // Simulation
        let start_time = time::now().to_timespec();
        let mut next_tick = 0;
        let mut plans_sent = false;
        for e in Events::events(window.clone()) {
            use piston::event;
            use piston::input;
            use piston::event::*;

            let e: event::Event<input::Input> = e;
        
            // Calculate a bunch of time stuff
            let current_time = time::now().to_timespec();
            let elapsed_time = current_time - start_time;
            let elapsed_seconds = (elapsed_time.num_milliseconds() as f64)/1000.0;
            
            if !self.final_ticks.is_some() && !self.player_ship.get(&self.bc).exploding && !plans_sent && elapsed_seconds >= 2.5 {
                // Send plans
                let packet = self.build_plans_packet(gui);
                self.client.send(&packet);
                plans_sent = true;
                println!("Sent plans at {}", elapsed_seconds);
            }
            
            if !self.final_ticks.is_some() {
                if plans_sent || self.player_ship.get(&self.bc).exploding {
                    if let Ok(packet) = self.client.try_receive() {
                        let ticked = self.handle_packet(gui, packet);
                        
                        if ticked && !self.final_ticks.is_some() {
                            // If the tick we got isn't the last tick, this turn is done.
                            println!("Finished turn at {}", elapsed_seconds);
                            break;
                        }
                    }
                }
            } else if elapsed_seconds >= 5.0 {
                println!("Finished turn because we're leaving this state");
                break;
            }
            
            // Calculate current tick
            let tick = (elapsed_time.num_milliseconds() as u32)/(1000/TICKS_PER_SECOND);
            
            // Simulate any new ticks
            if next_tick < 100 {
                for t in next_tick .. cmp::min(next_tick+tick-next_tick+1, 100) {
                    sim_events.apply_tick(&mut self.bc, t);
                }
                next_tick = tick+1;
            }
        
            // Forward events to GUI
            let gui_action = gui.event(&mut self.bc, &e, self.player_ship);
            
            if let Some(gui_action) = gui_action {
                match gui_action {
                    SpaceGuiAction::Chat(msg) => {
                        self.send_chat(msg);
                    },
                    SpaceGuiAction::Logout => {
                        self.send_logout();
                    },
                }
            }
            
            // Render GUI
            e.render(|args: &RenderArgs| {
                gl.draw(args.viewport(), |c, gl| {
                    gui.draw_simulating(&self.bc, &c, gl, glyph_cache, asset_store, &mut sim_effects, self.player_ship.get(&self.bc), elapsed_seconds, (1.0/60.0) + args.ext_dt);
                });
            });
        }
        
        // Simulate any remaining ticks
        for t in next_tick .. 100 {
            sim_events.apply_tick(&mut self.bc, t);
        }
        
        // After simulation
        self.bc.after_simulation();
        
        // Apply module stats
        self.bc.apply_module_stats();
        
        // Deactivate modules that can no longer be powered
        self.bc.deactivate_unpowerable_modules();
        
        // Set all the dead ships to exploding
        for ship in self.bc.ships_iter_mut() {
            if ship.state.get_hp() == 0 {
                ship.exploding = true;
            }
        }
        
        logging_out
    }
    
    fn build_plans_packet(&mut self, gui: &SpaceGui) -> OutPacket {
        let mut packet = OutPacket::new();
        packet.write(&ServerBattlePacket::Plan).unwrap();
        packet.write(&gui.plans).unwrap();
        packet
    }
    
    fn send_chat(&mut self, msg: String) {
        let mut packet = OutPacket::new();
        packet.write(&ServerBattlePacket::Chat(msg)).unwrap();
        self.client.send(&packet);
    }
    
    fn send_logout(&mut self) {
        let mut packet = OutPacket::new();
        packet.write(&ServerBattlePacket::Logout).unwrap();
        self.client.send(&packet);
    }
    
    fn handle_packet(&mut self, gui: &mut SpaceGui, mut packet: InPacket) -> bool {
        let battle_packet: ClientBattlePacket = packet.read().unwrap();
        
        match battle_packet {
            ClientBattlePacket::NewShipsPre => {
                self.new_ships_pre = Some(packet);
            },
            ClientBattlePacket::SimResults => {
                self.results = Some(packet);
            },
            ClientBattlePacket::NewShipsPost => {
                self.new_ships_post = Some(packet);
            },
            ClientBattlePacket::Tick(final_ticks) => {
                self.final_ticks = final_ticks;
                return true;
            },
            ClientBattlePacket::Chat(msg) => {
                gui.chat_gui.add_message(msg);
            },
        }
        
        false
    }
    
    fn handle_simulation_results(&mut self, packet: &mut InPacket) {
        // Results packet has both plans and results
        self.bc.read_results(packet);
    }
    
    fn handle_new_ships_packet(&mut self, gui: &mut SpaceGui, packet: &mut InPacket) {
        let ships_to_add: Vec<Ship> = packet.read().ok().expect("Failed to read ships to add from packet");
        let ships_to_remove: Vec<ShipIndex> = packet.read().ok().expect("Failed to read ships to remove from packet");
        
        let player_ship_id = self.player_ship.get(&self.bc).id;
        let player_hp = self.player_ship.get(&self.bc).state.get_hp();
        
        for ship in ships_to_remove.into_iter() {
            println!("Removing ship {:?}", ship);
            
            gui.on_ship_removed(self.player_ship, ship);
            self.bc.remove_ship(ship);
        }
    
        for ship in ships_to_add.into_iter() {
            println!("Got a new ship {:?}", ship.id);
            let ship_id = ship.id;
            if ship_id == player_ship_id {
                if player_hp == 0 {
                    println!("Replacing player's ship");
                    self.player_ship = ship.index;
                    gui.set_client_ship(&ship);
                    self.bc.add_ship(ship);
                }
            } else {
                println!("Trying to lock");
                let ship_index = ship.index;
                self.bc.add_ship(ship);
                gui.try_lock(ship_index);
            }
            println!("Added the ship");
        }

        println!("Finished readng new ships");
    }
}
