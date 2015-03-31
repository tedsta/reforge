use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use module::{ModulePlans, NetworkTarget};
use net::{ClientId, InPacket, OutPacket};
use ship::{ShipId, ShipNetworked, ShipRef};
use sim::SimEvents;

#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use asset_store::AssetStore;

// Time value of 1 tick in seconds
pub static TICKS_PER_SECOND: u32 = 20;

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
    
    pub fn add_ship(&mut self, ship: ShipRef) {
        self.ships_list.push(ship.clone());
        self.ships.insert(ship.borrow().id, ship.clone());
        let client_id = ship.borrow().client_id;
        if let Some(client_id) = client_id {
            self.ships_client_id.insert(client_id, ship);
        }
    }
    
    pub fn add_ships(&mut self, ships: Vec<ShipRef>) {
        for ship in ships {
            self.add_ship(ship);
        }
    }
    
    pub fn add_networked_ship(&mut self, ship: ShipNetworked) -> ShipRef {
        let (ship, targets) = ship.to_ship();
        let ship = Rc::new(RefCell::new(ship));
        
        self.add_ship(ship.clone());
        
        ship.borrow_mut().set_targets(self, &targets);
        
        ship
    }
    
    pub fn add_networked_ships(&mut self, ships: Vec<ShipNetworked>) {
        let ships: Vec<(ShipRef, Vec<(Option<NetworkTarget>, Option<NetworkTarget>)>)> =
            ships.into_iter().map(
                |s| {
                    let (ship, targets) = s.to_ship();
                    let ship = Rc::new(RefCell::new(ship));
                    
                    self.add_ship(ship.clone());
                    
                    (ship, targets)
                }
            ).collect();
        
        for (ship, targets) in ships {
            ship.borrow_mut().set_targets(self, &targets);
        }
    }
    
    pub fn remove_ship(&mut self, ship_id: ShipId) {
        self.on_ship_removed(ship_id);
    
        self.ships_list.retain(|ship| ship.borrow().id != ship_id);
        
        // TODO optimize this
        
        // Rebuild hash maps
        self.ships = HashMap::new();
        for ship in self.ships_list.iter() {
            self.ships.insert(ship.borrow().id, ship.clone());
        }
        
        self.ships_client_id = HashMap::new();
        for ship in self.ships_list.iter() {
            match ship.borrow().client_id {
                Some(client_id) => { self.ships_client_id.insert(client_id, ship.clone()); },
                None => {},
            }
        }
    }

    pub fn server_preprocess(&mut self) {
        for ship in self.ships_list.iter() {
            ship.borrow_mut().server_preprocess();
        }
    }
    
    pub fn before_simulation(&mut self, events: &mut SimEvents) {
        for ship in self.ships_list.iter() {
            ship.borrow().before_simulation(events, ship);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_plan_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects) {
        for ship in self.ships_list.iter() {
            ship.borrow().add_plan_effects(asset_store, effects, ship);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_simulation_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects) {
        for ship in self.ships_list.iter() {
            ship.borrow().add_simulation_effects(asset_store, effects, ship);
        }
    }
    
    pub fn after_simulation(&self) {
        for ship in self.ships_list.iter() {
            ship.borrow_mut().after_simulation();
        }
    }
    
    pub fn on_ship_removed(&self, ship_id: ShipId) {
        for ship in self.ships_list.iter() {
            ship.borrow_mut().on_ship_removed(ship_id);
        }
    }
    
    pub fn apply_module_plans(&self) {
        for ship in self.ships_list.iter() {
            ship.borrow_mut().apply_module_plans();
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        packet.write(&(self.ships.len() as u32));
        for ship in self.ships_list.iter() {
            packet.write(&ship.borrow().id);
            ship.borrow().write_results(packet);
        }
    }
    
    pub fn read_results(&self, packet: &mut InPacket) {
        let num_ships: u32 = packet.read().unwrap();
        for _ in 0 .. num_ships {
            let ship_id = packet.read().unwrap();
            let ship = self.get_ship(ship_id);
            
            ship.borrow_mut().read_results(self, packet);
        }
    }
}

// Packets sent from client to server
#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum ServerPacketId {
    Plan, // Player's plans
}

// Packets sent from server to client
#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum ClientPacketId {
    SimResults, // Calculated simulation results from server
}