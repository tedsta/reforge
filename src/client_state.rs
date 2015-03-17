use std::cell::RefCell;

use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;
use sdl2_window::Sdl2Window;

use asset_store::AssetStore;
use battle_state::BattleContext;
use client_battle_state::ClientBattleState;
use net::Client;
use sector_data::SectorData;

pub enum ClientState {
    JoinSector,
    Respawn,
}

pub fn run_client_state_manager(window: &RefCell<Sdl2Window>, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore, mut client: Client) {
    // Receive the star map
    let mut packet = client.receive();
    let sectors: Vec<SectorData> = packet.read().ok().expect("Failed to read star map");
    
    loop {
        // Receive the ships from the server
        let mut packet = client.receive();
        let my_ship = packet.read().ok().expect("Failed to read my Ship");
        let start_at_sim = packet.read().ok().expect("Failed to read start_at_sim from server");
        let ships = match packet.read() {
            Ok(ships) => ships,
            Err(e) => panic!("Unable to receive ships froms server: {}", e),
        };
        
        // Create the battle state
        let mut battle_context = BattleContext::new(ships);
        battle_context.add_ship(my_ship);
        let mut battle = ClientBattleState::new(&mut client, battle_context);

        battle.run(window, gl, glyph_cache, asset_store, sectors.clone(), start_at_sim);
    }
}