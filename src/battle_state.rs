use std::collections::HashMap;

use net::{ClientId, InPacket, OutPacket};
use ship::ShipRef;
use sim::SimEvents;

#[cfg(client)]
use sim::SimVisuals;
#[cfg(client)]
use asset_store::AssetStore;

// Time value of 1 tick in seconds
pub static TICKS_PER_SECOND: u32 = 20;

#[deriving(Encodable, Decodable)]
pub struct BattleContext {
    pub ships: HashMap<ClientId, ShipRef>,
}

impl BattleContext {
    pub fn new(ships: HashMap<ClientId, ShipRef>) -> BattleContext {
        BattleContext {
            ships: ships,
        }
    }
    
    pub fn get_ship<'a>(&'a self, client_id: ClientId) -> &'a ShipRef {
        match self.ships.find(&client_id) {
            Some(ship) => ship,
            None => fail!("No ship with client ID {}", client_id),
        }
    }
    
    pub fn server_preprocess(&mut self) {
        for ship in self.ships.values() {
            ship.borrow_mut().server_preprocess();
        }
    }
    
    pub fn before_simulation(&mut self, events: &mut SimEvents) {
        for ship in self.ships.values() {
            ship.borrow_mut().before_simulation(events);
        }
    }
    
    #[cfg(client)]
    pub fn add_plan_visuals(&mut self, asset_store: &AssetStore, visuals: &mut SimVisuals) {
        for ship in self.ships.values() {
            ship.borrow().add_plan_visuals(asset_store, visuals);
        }
    }
    
    #[cfg(client)]
    pub fn add_simulation_visuals(&mut self, asset_store: &AssetStore, visuals: &mut SimVisuals) {
        for ship in self.ships.values() {
            ship.borrow().add_simulation_visuals(asset_store, visuals);
        }
    }
    
    pub fn after_simulation(&mut self) {
        for ship in self.ships.values() {
            ship.borrow_mut().after_simulation();
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        for ship in self.ships.values() {
            ship.borrow().write_results(packet);
        }
    }
    
    pub fn read_results(&self, packet: &mut InPacket) {
        for ship in self.ships.values() {
            ship.borrow().read_results(packet);
        }
    }
    
    pub fn get_num_ships(&self) -> uint {
        self.ships.len()
    }
}

// Packets sent from client to server
#[deriving(Encodable, Decodable)]
pub enum ServerPacketId {
    Plan, // Player's plans
}

// Packets sent from server to client
#[deriving(Encodable, Decodable)]
pub enum ClientPacketId {
    SimResults, // Calculated simulation results from server
}