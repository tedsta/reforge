use std::cell::RefCell;
use std::old_io::IoResult;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::time::Duration;
use time;

use event::Events;
use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;
use sdl2_window::Sdl2Window;

use asset_store::AssetStore;
use battle_state::{BattleContext, ClientPacketId, ServerPacketId, TICKS_PER_SECOND};
use net::{Client, OutPacket};
use sector_data::SectorData;
use ship::{Ship, ShipId, ShipRef};
use sim::{SimEvents, SimEffects};
use space_gui::SpaceGui;

pub struct ClientBattleState<'a> {
    client: &'a mut Client,
    
    // Context holding all the things involved in this battle
    context: BattleContext,
    
    // The player's ship
    player_ship: ShipRef,
}

impl<'a> ClientBattleState<'a> {
    pub fn new(client: &'a mut Client, context: BattleContext) -> ClientBattleState<'a> {
        let player_ship = context.get_ship_by_client_id(client.get_id()).clone();
        ClientBattleState {
            client: client,
            context: context,
            player_ship: player_ship,
        }
    }
    
    pub fn run(&mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore, sectors: Vec<SectorData>, start_at_sim: bool) {
        use window::ShouldClose;
        use quack::Get;
    
        let ref mut gui = SpaceGui::new(asset_store, &self.context, sectors, self.player_ship.borrow().id);
    
        let ref mut sim_effects = SimEffects::new();
        
        // TODO display joining screen here
        
        // If we joined during a planning phase, wait for simulation to start
        if start_at_sim {
            while self.try_receive_new_ships(gui).is_err() {}
            while self.try_receive_simulation_results().is_err() {}
            
            self.run_simulation_phase(window, gl, glyph_cache, asset_store, gui, sim_effects);
            
            // Check if it's time to exit
            let ShouldClose(should_close) = window.borrow().get();
            if should_close { return; }
        }
    
        loop {
            ////////////////////////////////
            // Plan
            
            while self.try_receive_new_ships(gui).is_err() {}
            
            self.run_planning_phase(window, gl, glyph_cache, asset_store, gui, sim_effects);
            
            // Check if it's time to exit
            let ShouldClose(should_close) = window.borrow().get();
            if should_close { break; }
            
            ////////////////////////////////
            // Simulate
            
            self.run_simulation_phase(window, gl, glyph_cache, asset_store, gui, sim_effects);
            
            // Check if it's time to exit
            let ShouldClose(should_close) = window.borrow().get();
            if should_close { break; }
            
            // Check if player jumped
            if self.player_ship.borrow().target_sector.is_some() {
                break;
            }
        }
    }
    
    fn run_planning_phase(&mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore, gui: &mut SpaceGui, mut sim_effects: &mut SimEffects) {
        // Add planning effects
        sim_effects.reset();
        self.context.add_plan_effects(asset_store, &mut sim_effects);
        
        // Record start time
        let start_time = time::now().to_timespec();
        
        let mut plans_sent = false;
        
        // Run planning loop
        for e in Events::new(window) {
            use event;
            use input;
            use event::*;

            let e: event::Event<input::Input> = e;
        
            // Calculate a bunch of time stuff
            let current_time = time::now().to_timespec();
            let elapsed_time = current_time - start_time;
            let mut elapsed_seconds = (elapsed_time.num_milliseconds() as f64)/1000.0;
            if !plans_sent && elapsed_time.num_seconds() >= 5 {
                // Send plans
                let packet = self.build_plans_packet();
                self.client.send(&packet);
                plans_sent = true;
                println!("Sent plans at {}", elapsed_time.num_milliseconds());
            }
            
            // Break once we receive sim results
            if plans_sent && self.try_receive_new_ships(gui).is_ok() {
                println!("Receiving results");
                while self.try_receive_simulation_results().is_err() { }
                println!("Received results at {}", elapsed_time.num_milliseconds());
                break;
            } else {
            }
        
            // Forward events to GUI
            gui.event(&e, &self.player_ship);
            
            // Render GUI
            e.render(|args: &RenderArgs| {
                gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
                    gui.draw_planning(&c, gl, glyph_cache, asset_store, &mut sim_effects, self.player_ship.borrow_mut().deref_mut(), elapsed_seconds, (1.0/60.0) + args.ext_dt);
                });
            });
        }
    }
    
    fn run_simulation_phase(&mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore, gui: &mut SpaceGui, mut sim_effects: &mut SimEffects) {
        let mut sim_events = SimEvents::new();
            
        // Before simulation
        sim_effects.reset();
        self.context.before_simulation(&mut sim_events);
        self.context.add_simulation_effects(asset_store, &mut sim_effects);
        
        // Simulation
        let start_time = time::now().to_timespec();
        let mut next_tick = 0;
        for e in Events::new(window) {
            use event;
            use input;
            use event::*;

            let e: event::Event<input::Input> = e;
        
            // Calculate a bunch of time stuff
            let current_time = time::now().to_timespec();
            let elapsed_time = current_time - start_time;
            let elapsed_seconds = (elapsed_time.num_milliseconds() as f64)/1000.0;
            if elapsed_time.num_seconds() >= 5 {
                break;
            }
            
            // Calculate current tick
            let tick = (elapsed_time.num_milliseconds() as u32)/(1000/TICKS_PER_SECOND);
            
            // Simulate any new ticks
            for t in range(next_tick, next_tick + tick-next_tick+1) {
                sim_events.apply_tick(t);
            }
            next_tick = tick+1;
        
            // Forward events to GUI
            gui.event(&e, &self.player_ship);
            
            // Render GUI
            e.render(|args: &RenderArgs| {
                gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
                    gui.draw_simulating(&c, gl, glyph_cache, asset_store, &mut sim_effects, self.player_ship.borrow_mut().deref_mut(), elapsed_seconds, (1.0/60.0) + args.ext_dt);
                });
            });
        }
        
        // After simulation
        self.context.after_simulation();
        
        while self.try_receive_new_ships(gui).is_err() { }
    }
    
    fn build_plans_packet(&mut self) -> OutPacket {
        let mut packet = OutPacket::new();
        match packet.write(&ServerPacketId::Plan) {
            Ok(()) => {},
            Err(_) => panic!("Failed to write plan packet ID"),
        }

        packet.write(&self.player_ship.borrow().target_sector).ok().expect("Failed to write player's target sector");
        packet.write(&self.player_ship.borrow().get_module_plans()).ok().expect("Failed to write player's plans");

        packet
    }
    
    fn try_receive_simulation_results(&mut self) -> IoResult<()> {
        let mut packet = try!(self.client.try_receive());
        match packet.read::<ClientPacketId>() {
            Ok(ref id) if *id != ClientPacketId::SimResults => panic!("Expected SimResults, got something else"),
            Err(e) => panic!("Failed to read simulation results packet ID: {}", e),
            _ => {}, // All good!
        };
        
        // Results packet has both plans and results
        self.context.read_results(&mut packet);
        
        Ok(())
    }
    
    fn try_receive_new_ships(&mut self, gui: &mut SpaceGui) -> IoResult<()> {
        let mut packet = try!(self.client.try_receive());
        
        let ships_to_add: Vec<Ship> = packet.read().ok().expect("Failed to read ships to add from packet");
        let ships_to_remove: Vec<ShipId> = packet.read().ok().expect("Failed to read ships to remove from packet");
        
        for ship in ships_to_remove.into_iter() {
            println!("Removing ship {:?}", ship);
        
            gui.remove_lock(ship);
        
            self.context.remove_ship(ship);
        }
    
        for ship in ships_to_add.into_iter() {
            let ship = Rc::new(RefCell::new(ship));
            
            println!("Got a new ship {:?}", ship.borrow().id);
            
            if ship.borrow().id == self.player_ship.borrow().id {
                if self.player_ship.borrow().state.get_hp() == 0 {
                    println!("Replacing player's ship");
                    self.player_ship = ship.clone();
                    self.context.add_ship(ship);
                }
            } else {
                println!("Trying to lock");
                gui.try_lock(&ship);
                self.context.add_ship(ship);
            }
        }
        
        Ok(())
    }
}
