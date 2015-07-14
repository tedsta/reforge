use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter;
use std::slice;

use module::ModelStore;
use net::{ClientId, InPacket, OutPacket};
use ship::{Ship, ShipId, ShipIndex};
use sim::SimEvents;

#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use asset_store::AssetStore;

// Time value of 1 tick in seconds
pub static TICKS_PER_SECOND: u32 = 20;

pub fn tick_to_time(tick: u32) -> f64 {
    tick as f64 / TICKS_PER_SECOND as f64
}

pub struct BattleContext {
    pub ships_ship_id: HashMap<ShipId, usize>,
    pub ships_client_id: HashMap<ClientId, usize>,

    pub ships: Vec<Option<Ship>>,
    
    free_ship_indices: Vec<usize>,
}

impl BattleContext {
    pub fn new(ships: Vec<Option<Ship>>) -> BattleContext {
        // Build (ShipId -> ship index) map
        let mut ships_ship_id = HashMap::new();
        for (i, ship) in ships.iter().enumerate() {
            if let &Some(ref ship) = ship {
                ships_ship_id.insert(ship.id, i);
            }
        }
        
        // Build (ClientId -> ship index) map
        let mut ships_client_id = HashMap::new();
        for (i, ship) in ships.iter().enumerate() {
            if let &Some(ref ship) = ship {
                if let Some(client_id) = ship.client_id {
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
    
    fn ships_filter_map<'a>(x: &'a Option<Ship>) -> Option<&'a Ship> {
        x.as_ref()
    }
    
    fn ships_filter_map_mut<'a>(x: &'a mut Option<Ship>) -> Option<&'a mut Ship> {
        x.as_mut()
    }
    
    pub fn ships_iter<'a>(&'a self)
        -> iter::FilterMap<slice::Iter<'a, Option<Ship>>, fn(&'a Option<Ship>) -> Option<&'a Ship>>
    {
        self.ships.iter().filter_map(BattleContext::ships_filter_map)
    }
    
    pub fn ships_iter_mut<'a>(&'a mut self)
        -> iter::FilterMap<slice::IterMut<'a, Option<Ship>>, fn(&'a mut Option<Ship>) -> Option<&'a mut Ship>>
    {
        self.ships.iter_mut().filter_map(BattleContext::ships_filter_map_mut)
    }
    
    pub fn get_ship<'a>(&'a self, ship_id: ShipId) -> &'a Ship {
        match self.ships_ship_id.get(&ship_id) {
            Some(index) => {
                self.ships[*index].as_ref().expect("BattleContext::ships_ship_id points to invalid ship index")
            },
            None => panic!("No ship with ID {}", ship_id),
        }
    }
    
    pub fn get_ship_by_client_id<'a>(&'a self, client_id: ClientId) -> &'a Ship {
        match self.ships_client_id.get(&client_id) {
            Some(index) => {
                self.ships[*index].as_ref().expect("BattleContext::ships_client_id points to invalid ship index")
            },
            None => panic!("No ship with client ID {}", client_id),
        }
    }
    
    pub fn add_ship(&mut self, mut ship: Ship) -> ShipIndex {
        let index = self.ships.len();
        let ship_id = ship.id;
        let client_id = ship.client_id;
        
        ship.index = ShipIndex(index as u32);
        
        self.ships.push(Some(ship));
        self.ships_ship_id.insert(ship_id, index);
        
        if let Some(client_id) = client_id {
            self.ships_client_id.insert(client_id, index);
        }
        
        ShipIndex(index as u32)
    }
    
    pub fn add_ships(&mut self, ships: Vec<Ship>) {
        for ship in ships {
            self.add_ship(ship);
        }
    }
    
    pub fn remove_ship(&mut self, ship_index: ShipIndex) -> Ship {
        self.on_ship_removed(ship_index);
    
        self.ships[ship_index.to_usize()].take().expect("Tried to remove non-existant ship")
    }

    pub fn server_preprocess(&self, model_store: &ModelStore) {
        for ship in self.ships_iter() {
            ship.server_preprocess(self, model_store);
        }
    }
    
    pub fn before_simulation(&self, model_store: &ModelStore, events: &mut SimEvents) {
        for ship in self.ships_iter() {
            ship.before_simulation(self, model_store, events);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_plan_effects(&self, asset_store: &AssetStore, model_store: &ModelStore, effects: &mut SimEffects) {
        for ship in self.ships_iter() {
            ship.add_plan_effects(self, asset_store, model_store, effects);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_simulation_effects(&self, asset_store: &AssetStore, model_store: &ModelStore, effects: &mut SimEffects) {
        for ship in self.ships_iter() {
            ship.add_simulation_effects(self, asset_store, model_store, effects);
        }
    }
    
    pub fn after_simulation(&mut self) {
        for ship in self.ships_iter_mut() {
            ship.after_simulation();
        }
    }
    
    pub fn on_ship_removed(&mut self, ship_index: ShipIndex) {
        for ship in self.ships_iter_mut() {
            ship.on_ship_removed(ship_index);
        }
    }
    
    pub fn apply_module_stats(&mut self) {
        for ship in self.ships_iter_mut() {
            ship.apply_module_stats();
        }
    }
    
    pub fn deactivate_unpowerable_modules(&mut self) {
        for ship in self.ships_iter_mut() {
            ship.deactivate_unpowerable_modules();
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        packet.write(&(self.ships_iter().count() as u32));
        for ship in self.ships_iter() {
            packet.write(&ship.index);
            ship.write_results(packet);
        }
    }
    
    pub fn read_results(&mut self, packet: &mut InPacket) {
        let num_ships: u32 = packet.read().unwrap();
        for _ in 0 .. num_ships {
            let ship: ShipIndex = packet.read().unwrap();
            
            ship.get_mut(self).read_results(packet);
        }
    }
}