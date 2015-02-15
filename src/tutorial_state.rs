use time;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use event::Events;
use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;
use sdl2_window::Sdl2Window;

use ai::run_ai;
use asset_store::AssetStore;
use battle_state::{BattleContext, TICKS_PER_SECOND};
use ship::{Ship, ShipRef};
use sim::{SimEvents, SimEffects};
use space_gui::SpaceGui;

pub struct TutorialState {
    // Context holding all the things involved in this battle
    context: BattleContext,
    
    // The player's ship
    player_ship: ShipRef,
    
    // The enemy's ship
    enemy_ship: ShipRef,
}

impl TutorialState {
    pub fn new() -> TutorialState {
        let player_ship = Rc::new(RefCell::new(create_player_ship()));
        let enemy_ship = Rc::new(RefCell::new(create_enemy_ship()));
        
        let context = BattleContext::new(vec![player_ship.clone(), enemy_ship.clone()]);
    
        TutorialState {
            context: context,
            player_ship: player_ship,
            enemy_ship: enemy_ship,
        }
    }
    
    pub fn run(&mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore) {
        use window::ShouldClose;
        use quack::Get;
    
        let mut gui = SpaceGui::new(asset_store, &self.context, self.player_ship.borrow().id);
    
        let mut sim_effects = SimEffects::new();
    
        loop {
            ////////////////////////////////
            // Plan
            
            // Add planning effects
            sim_effects.reset();
            self.context.add_plan_effects(asset_store, &mut sim_effects);
            
            // Record start time
            let start_time = time::now().to_timespec();
            
            // Run planning loop
            for e in Events::new(window) {
                use std::old_io::timer::sleep;
                use std::time::Duration;
            
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
            
                // Forward events to GUI
                gui.event(&e, &self.player_ship);
                
                // Render GUI
                e.render(|args: &RenderArgs| {
                    gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
                        gui.draw_planning(&c, gl, glyph_cache, asset_store, &mut sim_effects, self.player_ship.borrow_mut().deref_mut(), elapsed_seconds, (1.0/60.0) + args.ext_dt);
                    });
                });
            }
            
            // Check if it's time to exit
            let ShouldClose(should_close) = window.borrow().get();
            if should_close { break; }
            
            // Apply player's module plans
            self.player_ship.borrow_mut().apply_module_plans();
            
            // Run enemy AI and apply module plans
            run_ai(self.enemy_ship.borrow_mut().deref_mut(), &vec![self.player_ship.clone()]);
            self.enemy_ship.borrow_mut().apply_module_plans();
            
            ////////////////////////////////
            // Simulate
            
            let mut sim_events = SimEvents::new();
            
            // Before simulation
            sim_effects.reset();
            self.context.server_preprocess();
            self.context.before_simulation(&mut sim_events);
            self.context.add_simulation_effects(asset_store, &mut sim_effects);
            
            // Simulation
            let start_time = time::now().to_timespec();
            let mut next_tick = 0;
            for e in Events::new(window) {
                use std::old_io::timer::sleep;
                use std::time::Duration;
            
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
            
            // Check if it's time to exit
            let ShouldClose(should_close) = window.borrow().get();
            if should_close { break; }
            
            // After simulation
            self.context.after_simulation();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Tutorial ship generation

fn create_player_ship() -> Ship {
    Ship::generate(0, "player".to_string(), 5)
}

fn create_enemy_ship() -> Ship {
    Ship::generate(1, "enemy".to_string(), 5)
}
