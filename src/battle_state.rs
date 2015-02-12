use std::collections::HashMap;

use module::ModulePlans;
use net::{ClientId, InPacket, OutPacket};
use ship::{ShipId, ShipRef};
use sim::SimEvents;

#[cfg(feature = "client")]
use sim::SimVisuals;
#[cfg(feature = "client")]
use asset_store::AssetStore;

// Time value of 1 tick in seconds
pub static TICKS_PER_SECOND: u32 = 20;

#[derive(RustcEncodable, RustcDecodable)]
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
        if let Some(client_id) = ship.borrow().client_id {
            self.ships_client_id.insert(client_id, ship);
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
            ship.borrow_mut().before_simulation(events);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals) {
        for ship in self.ships_list.iter() {
            ship.borrow().add_plan_visuals(asset_store, visuals, ship);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals) {
        for ship in self.ships_list.iter() {
            ship.borrow().add_simulation_visuals(asset_store, visuals, ship);
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
    
    pub fn write_plans(&self, packet: &mut OutPacket) {
        let mut module_plans = HashMap::new();
    
        for ship in self.ships_list.iter() {
            module_plans.insert(ship.borrow().id, ship.borrow().get_module_plans());
        }
        
        packet.write(&module_plans).ok().expect("Failed to write module plans");
    }
    
    pub fn read_plans(&self, packet: &mut InPacket, exclude_id: Option<ShipId>) {
        let module_plans: HashMap<ShipId, Vec<ModulePlans>> = packet.read().ok().expect("Failed to read module plans");
    
        for (ship_id, plans) in module_plans.iter() {
            if let Some(ref exclude_id) = exclude_id {
                if *ship_id == *exclude_id {
                    continue;
                }
            }
        
            let ship = self.get_ship(*ship_id);
            
            ship.borrow().set_module_plans(self, &plans);
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
        for _ in range(0, num_ships) {
            let ship_id = packet.read().unwrap();
            let ship = self.get_ship(ship_id);
            
            ship.borrow_mut().read_results(packet);
        }
    }
}

// Packets sent from client to server
#[derive(Show, PartialEq, RustcEncodable, RustcDecodable)]
pub enum ServerPacketId {
    Plan, // Player's plans
}

// Packets sent from server to client
#[derive(Show, PartialEq, RustcEncodable, RustcDecodable)]
pub enum ClientPacketId {
    SimResults, // Calculated simulation results from server
}