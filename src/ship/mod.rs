use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use battle_state::BattleContext;
use module::{IModule, ModuleBase, ModuleRef, Module, ModuleType, ModuleTypeStore};
use net::{ClientId, InPacket, OutPacket};
use self::ship_gen::generate_ship;
use sim::SimEvents;

#[cfg(client)]
use graphics::Context;
#[cfg(client)]
use opengl_graphics::Gl;

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
            hp: 0,
            total_module_hp: 0,
            thrust: 0,
            shields: 0,
            max_shields: 0,
        }
    }
    
    fn before_simulation(&mut self) {
        self.shields = 0;
    }
    
    pub fn deal_damage(&mut self, module: &mut ModuleBase, damage: u8) {
        // Can't deal more damage than there is HP
        let damage = cmp::min(self.total_module_hp, damage);
        
        // Get the amount of damage dealt to the module
        let damage = module.deal_damage(damage);
        
        // Adjust the ship's HP state
        self.total_module_hp -= damage;
        self.hp = self.total_module_hp/2;
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
        // Add to state hp
        self.state.total_module_hp += module.get_base().get_hp();
        self.state.hp = self.state.total_module_hp/2;
        
        // Setup module's index
        module.get_base_mut().index = self.modules.len() as u32;
        
        // Add the module
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
    
    #[cfg(client)]
    pub fn draw_module_hp(&self, context: &Context, gl: &mut Gl) {
        use graphics::*;
    
        for module in self.modules.iter() {
            let module = module.borrow();
            let module = module.get_base();
            
            let context = context
                .trans((module.x*48) as f64, (module.y*48) as f64);
        
            for i in range(0, module.get_power()) {
                let context = context
                    .rect(0.0, 4.0 * (i as f64), 8.0, 2.0);
                
                if i < module.get_hp() {
                    context
                        .rgb(0.0, 1.0, 0.0)
                        .draw(gl);
                } else {
                    context
                        .border_width(1.0)
                        .rgb(8.0, 0.3, 0.3)
                        .draw(gl);
                }
            }
            
            for i in range(module.get_power(), module.get_hp()) {
                context
                    .rect(0.0, 4.0 * (i as f64), 8.0, 2.0)
                    .rgb(1.0, 1.0, 0.0)
                    .draw(gl);
            }
            
            for i in range(cmp::max(module.get_power(), module.get_hp()), module.get_max_hp()) {
                context
                    .rect(0.0, 4.0 * (i as f64), 8.0, 2.0)
                    .border_width(1.0)
                    .rgb(0.8, 0.8, 0.3)
                    .draw(gl);
            }
        }
    }
}