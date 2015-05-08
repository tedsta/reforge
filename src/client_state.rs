use std::rc::Rc;
use std::cell::RefCell;

use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;
use sdl2_window::Sdl2Window;

use asset_store::AssetStore;
use battle_context::BattleContext;
use client_action::ClientAction;
use module::ModelStore;
use sector_client::ClientBattleState;
use station_client::StationClient;
use net::Client;
use sector_data::SectorData;
use ship::{Ship, ShipStored};

pub fn run_client_state_manager(window: &Rc<RefCell<Sdl2Window>>,
                                gl: &mut Gl,
                                glyph_cache: &mut GlyphCache,
                                asset_store: &AssetStore,
                                model_store: &ModelStore,
                                mut client: Client) {
    use client_action::ClientAction::*;

    // Receive the star map
    let mut packet = client.receive();
    let sectors: Vec<SectorData> = packet.read().ok().expect("Failed to read star map");
    
    loop {
        let mut client_action_packet = client.receive();
        let client_action: ClientAction = client_action_packet.read().ok().expect("Failed to read next ClientAction");
    
        match client_action {
            JoinSector => {
                // Receive the sector join packet
                let mut packet = client.receive();
                let my_ship: Ship = packet.read().ok().expect("Failed to read my Ship");
                let server_results_sent = packet.read().ok().expect("Failed to read server_results_sent from server");
                let ships: Vec<Option<Ship>> = packet.read().ok().expect("Unable to receive ships froms server");

                // Create the battle state
                let mut battle_context = BattleContext::new(ships);
                
                // Add the player's ship
                battle_context.add_ship(my_ship);
                
                let mut battle = ClientBattleState::new(&mut client, battle_context);

                battle.run(window, gl, glyph_cache, asset_store, sectors.clone(), server_results_sent);
                
                println!("I (client) left a sector");
            },
            JoinStation => {
                // Receive the station join packet
                let mut packet = client.receive();
                let my_ship: Option<ShipStored> = packet.read().ok().expect("Failed to read my Ship");
                
                let mut station_client = StationClient::new(&mut client, my_ship);
                
                station_client.run(window, gl, glyph_cache, asset_store, model_store, sectors.clone());
            },
            Logout => {
                break;
            },
        }
    }
}