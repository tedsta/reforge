use std::collections::HashMap;

use net::{ClientId, InPacket, OutPacket};
use ship::{ShipId, ShipRef};
use sim::SimEvents;

#[cfg(client)]
use sim::SimVisuals;
#[cfg(client)]
use asset_store::AssetStore;

// Time value of 1 tick in seconds
pub static TICKS_PER_SECOND: u32 = 20;

#[deriving(Encodable, Decodable)]
pub struct BattleContext {
    pub ships: HashMap<ShipId, ShipRef>,
    pub ships_client_id: HashMap<ClientId, ShipRef>,
    pub ships_list: Vec<ShipRef>,
}

impl BattleContext {
    pub fn new(ships: Vec<ShipRef>) -> BattleContext {
        let mut ships_map = HashMap::new();
        for ship in ships.iter() {
            ships_map.insert(ship.borrow().id, ship.clone());
        }
        
        let mut ships_client_id_map = HashMap::new();
        for ship in ships.iter() {
            match ship.borrow().client_id {
                Some(client_id) => { ships_client_id_map.insert(client_id, ship.clone()); },
                None => {},
            }
        }
    
        BattleContext {
            ships: ships_map,
            ships_client_id: ships_client_id_map,
            ships_list: ships,
        }
    }
    
    pub fn get_ship<'a>(&'a self, ship_id: ShipId) -> &'a ShipRef {
        match self.ships.get(&ship_id) {
            Some(ship) => ship,
            None => panic!("No ship with ID {}", ship_id),
        }
    }
    
    pub fn get_ship_by_client_id<'a>(&'a self, client_id: ClientId) -> &'a ShipRef {
        match self.ships_client_id.get(&client_id) {
            Some(ship) => ship,
            None => panic!("No ship with client ID {}", client_id),
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
            ship.borrow().add_plan_visuals(asset_store, visuals, ship);
        }
    }
    
    #[cfg(client)]
    pub fn add_simulation_visuals(&mut self, asset_store: &AssetStore, visuals: &mut SimVisuals) {
        for ship in self.ships.values() {
            ship.borrow().add_simulation_visuals(asset_store, visuals, ship);
        }
    }
    
    pub fn after_simulation(&mut self) {
        for ship in self.ships.values() {
            ship.borrow_mut().after_simulation();
        }
    }
    
    pub fn write_plans(&self, packet: &mut OutPacket) {
        packet.write(&(self.ships.len() as u32)).unwrap();
        for ship in self.ships.values() {
            packet.write(&ship.borrow().id).unwrap();
            ship.borrow().write_plans(packet);
        }
    }
    
    pub fn read_plans(&self, packet: &mut InPacket) {
        let num_ships: u32 = packet.read().unwrap();
        for _ in range(0, num_ships) {
            let ship_id = packet.read().unwrap();
            let ship = self.get_ship(ship_id);
            
            ship.borrow_mut().read_plans(self, packet);
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        packet.write(&(self.ships.len() as u32));
        for ship in self.ships.values() {
            packet.write(&ship.borrow().id);
            ship.borrow().write_results(packet);
        }
    }
    
    pub fn read_results(&self, packet: &mut InPacket) {
        let num_ships: u32 = packet.read().unwrap();
        for _ in range(0, num_ships) {
            let ship_id = packet.read().unwrap();
            let ship = self.get_ship(ship_id);
            
            ship.borrow().read_results(packet);
        }
    }
    
    pub fn get_num_ships(&self) -> uint {
        self.ships.len()
    }
}

// Packets sent from client to server
#[deriving(Show, PartialEq, Encodable, Decodable)]
pub enum ServerPacketId {
    Plan, // Player's plans
}

// Packets sent from server to client
#[deriving(Show, PartialEq, Encodable, Decodable)]
pub enum ClientPacketId {
    SimResults, // Calculated simulation results from server
}