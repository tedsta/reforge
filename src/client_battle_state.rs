use time;

use rsfml::graphics::{Font, RenderTarget, RenderTexture, RenderWindow, Color};

use asset_store::AssetStore;
use battle_state::{BattleContext, ClientPacketId, Plan, TICKS_PER_SECOND};
use net::{Client, OutPacket};
use sfml_renderer::SfmlRenderer;
use ship::ShipRef;
use sim::{SimEvents, SimVisuals};
use space_gui::SpaceGui;

pub struct ClientBattleState {
    client: Client,
    
    // Context holding all the things involved in this battle
    context: BattleContext,
    
    // The player's ship
    player_ship: ShipRef,
}

impl ClientBattleState {
    pub fn new(client: Client, context: BattleContext) -> ClientBattleState {
        let player_ship = context.get_ship(client.get_id()).clone();
        ClientBattleState {
            client: client,
            context: context,
            player_ship: player_ship,
        }
    }
    
    pub fn run(&mut self, window: &mut RenderWindow, asset_store: &AssetStore) {
        let font = Font::new_from_file("content/fonts/8bit.ttf").expect("Failed to load font");
        let mut gui = SpaceGui::new(&font, self.client.get_id(), &self.context);
    
        let mut sim_visuals = SimVisuals::new();
    
        loop {
            ////////////////////////////////
            // Plan
            
            // Add planning visuals
            self.context.add_plan_visuals(asset_store, &mut sim_visuals);
            
            let start_time = time::now().to_timespec();
            while window.is_open() {
                let current_time = time::now().to_timespec();
                let elapsed_time = current_time - start_time;
                if elapsed_time.num_seconds() >= 5 {
                    break;
                }
                
                // Update gui
                gui.update(window, self.player_ship.borrow().deref());
                
                // Do planning stuff
                self.plan();
                
                // Render
                window.clear(&Color::transparent());
                self.draw_planning(window, asset_store, &mut sim_visuals, &mut gui);
                window.display();
            }
        
            // Send plans
            let packet = self.build_plans_packet();
            self.client.send(&packet);
            
            // Wait for simulation results
            self.receive_simulation_results();
            
            ////////////////////////////////
            // Simulate
            
            let mut sim_events = SimEvents::new();
            
            // Before simulation
            self.context.before_simulation(&mut sim_events);
            self.context.add_simulation_visuals(asset_store, &mut sim_visuals);
            
            // Simulation
            let start_time = time::now().to_timespec();
            let mut last_time = time::now().to_timespec();
            let mut next_tick = 0;
            while window.is_open() {
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
                
                // Calculate current tick
                let tick = (elapsed_time.num_milliseconds() as u32)/(1000/TICKS_PER_SECOND);
                
                // Calculate elapsed time in seconds as f32
                let elapsed_seconds = (elapsed_time.num_milliseconds() as f32)/1000f32;
                
                // Prepare last_time for next frame
                last_time = current_time;
                
                // Update gui
                gui.update(window, self.player_ship.borrow().deref());
                
                // Simulate any new ticks
                for t in range(next_tick, next_tick + tick-next_tick+1) {
                    sim_events.apply_tick(t);
                }
                next_tick = tick+1;
                
                // Render
                window.clear(&Color::transparent());
                self.draw_simulating(window, asset_store, &mut sim_visuals, &mut gui, elapsed_seconds);
                window.display();
            }
            
            // After simulation
            self.context.after_simulation();
            
            // Clear the visuals
            sim_visuals.clear();
        }
    }
    
    fn plan(&mut self) {
    }
    
    fn build_plans_packet(&mut self) -> OutPacket {
        let mut packet = OutPacket::new();
        match packet.write(&Plan) {
            Ok(()) => {},
            Err(e) => panic!("Failed to write plan packet ID: {}", e),
        }
        
        self.player_ship.borrow().write_plans(&mut packet);

        packet
    }
    
    fn receive_simulation_results(&mut self) {
        let mut packet = self.client.receive();
        let id: ClientPacketId = match (packet.read()) {
            Ok(id) => id,
            Err(e) => panic!("Failed to read simulation results packet ID: {}", e)
        };
        
        self.context.read_results(&mut packet);
    }
    
    fn draw_planning(&self, target: &RenderTarget, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, gui: &mut SpaceGui) {
        {
            let mut renderer = SfmlRenderer::new(target, asset_store);
            renderer.translate(100.0, 100.0);
            
            // Draw sim visuals for player ship
            sim_visuals.draw(&renderer, self.player_ship.borrow().id, 0.0);
        }
        
        let renderer = SfmlRenderer::new(target, asset_store);
        
        // Draw GUI
        gui.draw_planning(&renderer, asset_store, sim_visuals, self.player_ship.borrow().deref());
    }
    
    fn draw_simulating(&self, target: &RenderTarget, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, gui: &mut SpaceGui, time: f32) {
        {
            let mut renderer = SfmlRenderer::new(target, asset_store);
            renderer.translate(100.0, 100.0);
        
            // Draw sim visuals for player ship
            sim_visuals.draw(&renderer, self.player_ship.borrow().id, time);
        }
        
        let renderer = SfmlRenderer::new(target, asset_store);
        
        // Draw GUI
        gui.draw_simulating(&renderer, asset_store, sim_visuals, self.player_ship.borrow().deref(), time);
    }
}