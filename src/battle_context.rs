use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter;
use std::slice;

use module::{ModuleBase, Target};
use net::{ClientId, InPacket, OutPacket};
use ship::{ShipId, ShipIndex, ShipRef};
use sim::SimEvents;

#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use asset_store::AssetStore;

// Time value of 1 tick in seconds
pub static TICKS_PER_SECOND: u32 = 20;

pub struct BattleContext {
    pub ships_ship_id: HashMap<ShipId, usize>,
    pub ships_client_id: HashMap<ClientId, usize>,

    pub ships: Vec<Option<ShipRef>>,
    //pub modules: Vec<ModuleRef>,
    
    free_ship_indices: Vec<usize>,
}

impl BattleContext {
    pub fn new(ships: Vec<Option<ShipRef>>) -> BattleContext {
        // Build (ShipId -> ship index) map
        let mut ships_ship_id = HashMap::new();
        for (i, ship) in ships.iter().enumerate() {
            if let &Some(ref ship) = ship {
                ships_ship_id.insert(ship.borrow().id, i);
            }
        }
        
        // Build (ClientId -> ship index) map
        let mut ships_client_id = HashMap::new();
        for (i, ship) in ships.iter().enumerate() {
            if let &Some(ref ship) = ship {
                if let Some(client_id) = ship.borrow().client_id {
                    ships_client_id.insert(client_id, i);
                }
            }
        }
    
        BattleContext {
            ships_ship_id: ships_ship_id,
            ships_client_id: ships_client_id,
            ships: ships,
            free_ship_indices: vec!(),
        }
    }
    
    fn ships_filter_map<'a>(x: &'a Option<ShipRef>) -> Option<&'a ShipRef> {
        x.as_ref()
    }
    
    pub fn ships_iter<'a>(&'a self)
        -> iter::FilterMap<slice::Iter<'a, Option<ShipRef>>, fn(&'a Option<ShipRef>) -> Option<&'a ShipRef>>
    {
        self.ships.iter().filter_map(BattleContext::ships_filter_map)
    }
    
    pub fn get_ship<'a>(&'a self, ship_id: ShipId) -> &'a ShipRef {
        match self.ships_ship_id.get(&ship_id) {
            Some(index) => {
                self.ships[*index].as_ref().expect("BattleContext::ships_ship_id points to invalid ship index")
            },
            None => panic!("No ship with ID {}", ship_id),
        }
    }
    
    pub fn get_ship_by_client_id<'a>(&'a self, client_id: ClientId) -> &'a ShipRef {
        match self.ships_client_id.get(&client_id) {
            Some(index) => {
                self.ships[*index].as_ref().expect("BattleContext::ships_client_id points to invalid ship index")
            },
            None => panic!("No ship with client ID {}", client_id),
        }
    }
    
    pub fn add_ship(&mut self, ship: ShipRef) {
        let index = self.ships.len();
        let client_id = ship.borrow().client_id;
        
        ship.borrow_mut().index = ShipIndex(index as u32);
        
        self.ships.push(Some(ship.clone()));
        self.ships_ship_id.insert(ship.borrow().id, index);
        
        if let Some(client_id) = client_id {
            self.ships_client_id.insert(client_id, index);
        }
    }
    
    pub fn add_ships(&mut self, ships: Vec<ShipRef>) {
        for ship in ships {
            self.add_ship(ship);
        }
    }
    
    pub fn remove_ship(&mut self, ship_index: ShipIndex) {
        self.on_ship_removed(ship_index);
        
        if self.ships[ship_index.to_usize()].is_none() {
            panic!("Tried to remove non-existant ship");
        }
    
        self.ships[ship_index.to_usize()] = None;
    }

    pub fn server_preprocess(&mut self) {
        for ship in self.ships_iter() {
            ship.borrow_mut().server_preprocess(self);
        }
    }
    
    pub fn before_simulation(&mut self, events: &mut SimEvents) {
        for ship in self.ships_iter() {
            ship.borrow().before_simulation(self, events);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_plan_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects) {
        for ship in self.ships_iter() {
            ship.borrow().add_plan_effects(asset_store, effects, ship);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_simulation_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects) {
        for ship in self.ships_iter() {
            ship.borrow().add_simulation_effects(self, asset_store, effects, ship);
        }
    }
    
    pub fn after_simulation(&self) {
        for ship in self.ships_iter() {
            ship.borrow_mut().after_simulation();
        }
    }
    
    pub fn on_ship_removed(&self, ship_index: ShipIndex) {
        for ship in self.ships_iter() {
            ship.borrow_mut().on_ship_removed(ship_index);
        }
    }
    
    pub fn apply_module_plans(&self) {
        for ship in self.ships_iter() {
            ship.borrow_mut().apply_module_plans();
        }
    }
    
    pub fn apply_module_stats(&self) {
        for ship in self.ships_iter() {
            ship.borrow_mut().apply_module_stats();
        }
    }
    
    pub fn deactivate_unpowerable_modules(&self) {
        for ship in self.ships_iter() {
            ship.borrow_mut().deactivate_unpowerable_modules();
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        packet.write(&(self.ships_iter().count() as u32));
        for ship in self.ships_iter() {
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
    Tick,
    LastTick,
}