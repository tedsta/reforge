use time;

use opengl_graphics::Gl;
use piston::Window;

use asset_store::AssetStore;
use battle_state::{BattleContext, ClientPacketId, Plan, TICKS_PER_SECOND};
use net::{Client, OutPacket};
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
    
    pub fn run<W: Window>(&mut self, window: &mut W, gl: &mut Gl, asset_store: &AssetStore) {
        let mut gui = SpaceGui::new(asset_store, &self.context, self.client.get_id());
    
        let mut sim_visuals = SimVisuals::new();
    
        loop {
            ////////////////////////////////
            // Plan
            
            // Add planning visuals
            self.context.add_plan_visuals(asset_store, &mut sim_visuals);
            
            // Store mouse coordinates
            let mut (mouse_x, mouse_y) = (0, 0);
            
            // Record start time
            let start_time = time::now().to_timespec();
            
            // Run planning loop
            for e in Events::new(window) {
                // Keep track of time, break when planning is done
                let current_time = time::now().to_timespec();
                let elapsed_time = current_time - start_time;
                if elapsed_time.num_seconds() >= 5 {
                    break;
                }
            
                // Forward events to GUI
                gui.event(&e, self.player_ship.borrow().deref());
                
                // Render GUI
                e.render(|r_args| gui.draw_planning(r_args, gl, asset_store, &mut sim_visuals, self.player_ship.borrow().deref()));
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
            for e in Events::new(window) {
                // Keep track of time, break when simulation is done
                let current_time = time::now().to_timespec();
                let elapsed_time = current_time - start_time;
                if elapsed_time.num_seconds() >= 5 {
                    break;
                }
                
                // Calculate current tick
                let tick = (elapsed_time.num_milliseconds() as u32)/(1000/TICKS_PER_SECOND);
                
                // Calculate elapsed time in seconds as f32
                let elapsed_seconds = (elapsed_time.num_milliseconds() as f32)/1000f32;
                
                // Prepare last_time for next frame
                last_time = current_time;
                
                // Simulate any new ticks
                for t in range(next_tick, next_tick + tick-next_tick+1) {
                    sim_events.apply_tick(t);
                }
                next_tick = tick+1;
            
                // Forward events to GUI
                gui.event(&e, self.player_ship.borrow().deref());
                
                // Render GUI
                e.render(|r_args| gui.draw_simulating(r_args, gl, asset_store, &mut sim_visuals, self.player_ship.borrow().deref(), elapsed_seconds));
            }
            
            // After simulation
            self.context.after_simulation();
            
            // Clear the visuals
            sim_visuals.clear();
        }
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
}