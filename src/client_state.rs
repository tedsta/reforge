use std::rc::Rc;
use std::cell::RefCell;

use ggez::{Context, GameResult};

use asset_store::AssetStore;
use battle_context::BattleContext;
//use chat::ChatGui;
use client_action::ClientAction;
use client_context::ReforgeClientContext;
use game_state;
use module::ModelStore;
use sector_client::ClientBattleState;
use star_map::station::StationClient;
use net::Client;
use sector_data::SectorData;
use ship::{Ship, ShipStored};

pub fn run_client_state_manager(
    gtx: &mut ReforgeClientContext, ctx: &mut Context)
    -> GameResult<()>
{
    use client_action::ClientAction::*;
    
    //let ref mut chat_gui = ChatGui::new();
    
    loop {
        let mut client_action_packet = gtx.client.receive();
        let client_action: ClientAction =
            client_action_packet.read().ok().expect("Failed to read next ClientAction");
    
        match client_action {
            JoinSector => {
                // Receive the sector join packet
                let mut packet = gtx.client.receive();
                let my_ship: Ship =
                    packet.read().ok().expect("Failed to read my Ship");
                let server_results_sent =
                    packet.read().ok().expect("Failed to read server_results_sent from server");
                let ships: Vec<Option<Ship>> =
                    packet.read().ok().expect("Unable to receive ships froms server");

                // Create the battle state
                let mut battle_context = BattleContext::new(ships);
                
                // Add the player's ship
                battle_context.add_ship(my_ship);
                
                let mut battle = ClientBattleState::new(gtx, battle_context, ctx)?;

                battle.run(
                    gtx, ctx, /*chat_gui,*/server_results_sent);
                
                println!("I (client) left a sector");
            },
            JoinStation => {
                println!("Join station");
                // Receive the station join packet
                let mut packet = gtx.client.receive();
                let my_ship: Option<ShipStored> =
                    packet.read().expect("Failed to read my Ship");
                
                let mut station_client = StationClient::new(gtx, my_ship/*, chat_gui*/)?;
                
                station_client.run(gtx, ctx)?;
                println!("Leave station");
            },
            Logout => {
                break;
            },
            _ => { break; },
        }
    }

    Ok(())
}
