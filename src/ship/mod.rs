use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use battle_state::BattleContext;
use module::{IModule, ModuleBase, ModuleRef, Module, ModuleType, ModuleTypeStore};
use net::{ClientId, InPacket, OutPacket};
use self::ship_gen::generate_ship;
use sim::SimEvents;

#[cfg(client)]
use sim::SimVisuals;
#[cfg(client)]
use asset_store::AssetStore;

// Use the ship_gen module privately here
mod ship_gen;

// Holds everything about the ship's damage, capabilities, etc.
#[deriving(Encodable, Decodable)]
pub struct ShipState {
    hp: u8,
    total_module_hp: u8, // Sum of HP of all modules, used to recalculate HP when damaged
    pub thrust: u8,
    pub shields: u8,
    pub max_shields: u8,
}

impl ShipState {
    pub fn new() -> ShipState {
        ShipState {
            hp: 10,
            total_module_hp: 10,
            thrust: 0,
            shields: 0,
            max_shields: 0,
        }
    }
    
    pub fn deal_damage(&mut self, module: &mut ModuleBase, mut damage: u8) {
        if self.hp > 0 {
            damage = cmp::min(self.hp, damage);
            self.hp -= damage;
            //self.hp = self.total_module_hp/2;
            module.deal_damage(damage);
        }
    }
    
    pub fn get_hp(&self) -> u8 {
        self.hp
    }
}

pub type ShipRef = Rc<RefCell<Ship>>;

// Type for the ID of a ship
pub type ShipId = u64;

#[deriving(Encodable, Decodable)]
pub struct Ship {
    pub id: ShipId,
    pub client_id: Option<ClientId>,
    pub state: ShipState,
    pub modules: Vec<ModuleRef>,
}

impl Ship {
    pub fn new(id: ShipId) -> Ship {
        Ship {
            id: id,
            client_id: None,
            state: ShipState::new(),
            modules: vec!(),
        }
    }
    
    pub fn generate(mod_store: &ModuleTypeStore, id: ShipId) -> Ship {
        generate_ship(mod_store, id)
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module(&mut self, mut module: Module) -> bool {
        module.get_base_mut().index = self.modules.len() as u32;
        self.modules.push(Rc::new(RefCell::new(module)));
        true
    }
    
    pub fn server_preprocess(&mut self) {
        for module in self.modules.iter() {
            module.borrow_mut().server_preprocess(&mut self.state);
        }
    }
    
    pub fn before_simulation(&mut self, events: &mut SimEvents) {
        for module in self.modules.iter() {
            module.borrow_mut().before_simulation(&mut self.state, &mut events.create_adder(module.clone()));
        }
    }
    
    #[cfg(client)]
    pub fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship_ref: &ShipRef) {
        for module in self.modules.iter() {
            module.borrow().add_plan_visuals(asset_store, visuals, ship_ref);
        }
    }
    
    #[cfg(client)]
    pub fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship_ref: &ShipRef) {
        for module in self.modules.iter() {
            module.borrow().add_simulation_visuals(asset_store, visuals, ship_ref);
        }
    }
    
    pub fn after_simulation(&mut self) {
        for module in self.modules.iter() {
            module.borrow_mut().after_simulation(&mut self.state);
        }
    }
    
    pub fn write_plans(&self, packet: &mut OutPacket) {
        for module in self.modules.iter() {
            module.borrow().write_plans(packet);
        }
    }
    
    pub fn read_plans(&self, context: &BattleContext, packet: &mut InPacket) {
        for module in self.modules.iter() {
            module.borrow_mut().read_plans(context, packet);
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        for module in self.modules.iter() {
            module.borrow().write_results(packet);
        }
    }
    
    pub fn read_results(&self, packet: &mut InPacket) {
        for module in self.modules.iter() {
            module.borrow_mut().read_results(packet);
        }
    }
}