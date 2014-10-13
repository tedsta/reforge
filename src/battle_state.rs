use std::collections::HashMap;

use net::ClientId;
use ship::{Ship, ShipIndex};
use sim_element::SimElement;

// Time value of 1 tick in seconds
pub static TICKS_PER_SECOND: u32 = 20;

#[deriving(Encodable, Decodable)]
pub struct BattleContext {
    pub ships: Vec<Ship>,
    ship_client_ids: HashMap<ClientId, ShipIndex>,
}

impl BattleContext {
    pub fn new(ships: HashMap<ClientId, Ship>) -> BattleContext {
        let mut context = BattleContext {
            ships: vec!(),
            ship_client_ids: HashMap::new(),
        };
        
        for (client_id, ship) in ships.move_iter() {
            context.ship_client_ids.insert(client_id, ship.index);
            context.add_ship(ship);
        }
        
        context
    }
    
    pub fn add_ship(&mut self, mut ship: Ship) {
        ship.index.index = Some(self.ships.len() as u16);
        for module in ship.modules.iter() {
            module.borrow_mut().get_base_mut().ship = Some(ship.index);
        }
        self.ships.push(ship);
    }
    
    pub fn get_ship<'a>(&'a self, ship: &ShipIndex) -> Option<&'a Ship> {
        let index = match ship.index {
            Some(index) => index,
            None => return None,
        } as uint;
        if index >= self.ships.len() {
            return None;
        }
        Some(&self.ships[index])
    }
    
    pub fn apply_to_sim_elements(&self, f: |&mut SimElement|) {
        for ship in self.ships.iter() {
            for module in ship.modules.iter() {
                f(module.borrow_mut().deref_mut() as &mut SimElement);
            }
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