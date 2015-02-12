use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use battle_state::BattleContext;
use module;
use module::{IModule, IModuleRef, IModuleStored, Module, ModuleBase, ModuleBox, ModuleRef, ModuleStoredBox};
use net::{ClientId, InPacket, OutPacket};
use self::ship_gen::generate_ship;
use sim::SimEvents;

#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

#[cfg(feature = "client")]
use sim::SimVisuals;
#[cfg(feature = "client")]
use asset_store::AssetStore;
#[cfg(feature = "client")]
use space_gui::ModuleIcons;

// Use the ship_gen module privately here
mod ship_gen;

// Holds everything about the ship's damage, capabilities, etc.
#[derive(RustcEncodable, RustcDecodable)]
pub struct ShipState {
    hp: u8,
    total_module_hp: u8, // Sum of HP of all modules, used to recalculate HP when damaged
    pub power: u8,
    pub plan_power: u8, // Keeps track of power for planning
    pub thrust: u8,
    pub shields: u8,
    pub max_shields: u8,
}

impl ShipState {
    pub fn new() -> ShipState {
        ShipState {
            hp: 0,
            total_module_hp: 0,
            power: 0,
            plan_power: 0,
            thrust: 0,
            shields: 0,
            max_shields: 0,
        }
    }
    
    pub fn can_activate_module(&self, module: &ModuleBase) -> bool {
        if module.can_activate() && self.power >= module.get_power() {
            true
        } else {
            false
        }
    }
    
    pub fn can_plan_activate_module(&self, module: &ModuleBase) -> bool {
        if module.can_plan_activate() && self.plan_power >= module.get_power() {
            true
        } else {
            false
        }
    }
    
    pub fn activate_module(&mut self, module: &mut ModuleBase) {
        self.plan_power -= module.get_power();
        module.plan_powered = true;
    }
    
    pub fn deactivate_module(&mut self, module: &mut ModuleBase) {
        self.plan_power += module.get_power();
        module.plan_powered = false;
    }
    
    fn pre_before_simulation(&mut self) {
        self.shields = 0;
    }
    
    pub fn deal_damage(&mut self, modules: &Vec<ModuleRef>, module: &mut ModuleBox, damage: u8) {
        // Can't deal more damage than there is HP
        let damage = cmp::min(self.hp, damage);
        
        // Get if module was active before damage
        let was_active = module.get_base().is_active();
        
        if self.shields > 0 {
            self.shields -= cmp::min(self.shields, damage);
        } else {
            // Get the amount of damage dealt to the module
            let damage = module.get_base_mut().deal_damage(damage);
            
            // Adjust the ship's HP state
            self.hp -= damage;
            
            // If the module was active and can no longer be active, deactivate
            if !module.get_base().is_active() {
                if was_active {
                    // Module just got deactivated
                    self.add_power(module.get_base().get_power());
                    module.get_base_mut().plan_powered = false;
                    module.get_base_mut().powered = false;
                    module.on_deactivated(self, modules);
                } else if module.get_base_mut().plan_powered && !module.get_base_mut().can_activate() {
                    self.deactivate_module(module.get_base_mut());
                }
            }
        }
    }
    
    pub fn add_power(&mut self, power: u8) {
        self.power += power;
        self.plan_power += power;
    }
    
    pub fn remove_power(&mut self, power: u8, modules: &Vec<ModuleRef>) {
        for module in modules.iter() {
            if power <= self.power && power <= self.plan_power {
                break;
            } else {
                // Attempt to borrow the module
                match module.try_borrow_mut() {
                    Some(mut module) => {
                        if module.get_base().get_power() > 0 {
                            if power > self.power && module.get_base().powered {
                                self.add_power(module.get_base().get_power());
                                module.get_base_mut().plan_powered = false;
                                module.get_base_mut().powered = false;
                                module.on_deactivated(self, modules);
                            } else if power > self.plan_power && !module.get_base().powered && module.get_base().plan_powered {
                                self.deactivate_module(module.get_base_mut());
                            }
                        }
                    },
                    None => {},
                }
            }
        }
        
        self.power -= power;
        self.plan_power -= power;
    }
    
    pub fn add_shields(&mut self, shields: u8) {
        self.max_shields += shields;
    }
    
    pub fn remove_shields(&mut self, shields: u8) {
        self.max_shields -= shields;
        self.shields = cmp::min(self.shields, self.max_shields);
    }
    
    pub fn get_hp(&self) -> u8 {
        self.hp
    }
}

pub type ShipRef = Rc<RefCell<Ship>>;

// Type for the ID of a ship
pub type ShipId = u64;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Ship {
    pub id: ShipId,
    pub name: String,
    pub client_id: Option<ClientId>,
    pub state: ShipState,
    pub modules: Vec<ModuleRef>,
    
    // Ship dimensions in module blocks
    width: u8,
    height: u8,
    
    pub level: u8, // TODO: This is very temporary only for IC US semifinals
}

impl Ship {
    pub fn new(id: ShipId, name: String, level: u8) -> Ship {
        Ship {
            id: id,
            name: name,
            client_id: None,
            state: ShipState::new(),
            modules: vec!(),
            
            width: 0,
            height: 0,
            
            level: level,
        }
    }
    
    pub fn generate(id: ShipId, name: String, level: u8) -> Ship {
        generate_ship(id, name, level)
    }
    
    pub fn get_width(&self) -> u8 {
        self.width
    }
    
    pub fn get_height(&self) -> u8 {
        self.height
    }
    
    pub fn is_space_free(&self, x: u8, y: u8, width: u8, height: u8) -> bool {
        for module in self.modules.iter() {
            let module = (*module).borrow();
            let base = module.get_base();
            
            if base.x + base.width > x && base.x < x + width && base.y + base.height > y && base.y < y + height {
                return false;
            }
        }
        
        true
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module<M>(&mut self, mut module: Module<M>) -> bool
        where M: IModule + Clone + 'static
    {
        // Add to state hp
        self.state.total_module_hp += module.get_base().get_hp();
        self.state.hp = self.state.total_module_hp/2;
        
        // Modify the ship's dimensions
        self.width = cmp::max(self.width, module.get_base().x + module.get_base().width);
        self.height = cmp::max(self.height, module.get_base().x + module.get_base().height);
        
        // Setup module's index
        module.get_base_mut().index = self.modules.len() as u32;
        
        // Activate module if can
        if module.get_base().is_active() {
            module.on_activated(&mut self.state, &self.modules);
        }
        
        // Add the module
        self.modules.push(Rc::new(RefCell::new(ModuleBox::new(module))));
        true
    }
    
    pub fn deal_damage(&mut self, module: &mut ModuleBox, damage: u8) {
        self.state.deal_damage(&self.modules, module, damage);
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
    
    #[cfg(feature = "client")]
    pub fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship_ref: &ShipRef) {
        for module in self.modules.iter() {
            module.borrow().add_plan_visuals(asset_store, visuals, ship_ref);
        }
    }
    
    #[cfg(feature = "client")]
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
    
    pub fn on_ship_removed(&self, ship_id: ShipId) {
        for module in self.modules.iter() {
            module.borrow_mut().on_ship_removed(ship_id);
        }
    }
    
    pub fn apply_module_plans(&mut self) {
        println!("Applying plans");
        for module in self.modules.iter() {
            let mut module = module.borrow_mut();
            
            if module.get_base().plan_powered != module.get_base().powered {
                if module.get_base().can_activate() && self.state.power >= module.get_base().get_power() {
                    module.get_base_mut().powered = true;
                    self.state.power -= module.get_base().get_power();
                    module.on_activated(&mut self.state, &self.modules);
                } else {
                    module.get_base_mut().powered = false;
                    self.state.power += module.get_base().get_power();
                    module.on_deactivated(&mut self.state, &self.modules);
                }
                
                module.get_base_mut().plan_powered = module.get_base().powered;
            }
        }
    }
    
    pub fn get_module_plans(&self) -> Vec<module::ModulePlans> {
        self.modules.iter().map(|m| m.borrow().get_base().get_plans()).collect()
    }
    
    pub fn set_module_plans(&self, context: &BattleContext, plans: &Vec<module::ModulePlans>) {
        for (module, plans) in self.modules.iter().zip(plans.iter()) {
            module.borrow_mut().get_base_mut().set_plans(context, plans);
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        packet.write(&self.state.power);
        for module in self.modules.iter() {
            let module = module.borrow();
        
            // TODO: fix this ugliness when inheritance is a thing in Rust
            // Write the base results
            packet.write(&module.get_base().powered);
        
            module.write_results(packet);
        }
    }
    
    pub fn read_results(&mut self, packet: &mut InPacket) {
        self.state.power = packet.read().ok().expect("Failed to read ShipState power");
        for module in self.modules.iter() {
            let mut module = module.borrow_mut();
            
            // TODO: fix this ugliness when inheritance is a thing in Rust
            // Read the base results
            module.get_base_mut().powered = packet.read().ok().expect("Failed to read ModuleBase powered");
        
            module.read_results(packet);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn draw_module_hp(&self, context: &Context, gl: &mut Gl) {
        use quack::Set;
        use graphics::*;
    
        for module in self.modules.iter() {
            let module = module.borrow();
            let module = module.get_base();
            
            let context = context.trans((module.x as f64) * 48.0, (module.y as f64) * 48.0);
            
            let hp_rect = Rectangle::new([0.0, 1.0, 0.0, 1.0]);
            let hp_dmg_rect = Rectangle::border([0.8, 0.3, 0.3, 1.0], 1.0);
            let armor_rect = Rectangle::new([1.0, 1.0, 0.0, 1.0]);
            let armor_dmg_rect = Rectangle::border([0.8, 0.8, 0.3, 1.0], 1.0);
        
            for i in range(0, module.get_min_hp()) {
                if i < module.get_hp() {
                    hp_rect.draw([0.0, 4.0 * (i as f64), 8.0, 2.0], &context, gl);
                } else {
                    hp_dmg_rect.draw([0.0, 4.0 * (i as f64), 8.0, 2.0], &context, gl);
                }
            }
            
            for i in range(module.get_min_hp(), module.get_hp()) {
                armor_rect.draw([0.0, 4.0 * (i as f64), 8.0, 2.0], &context, gl);
            }
            
            for i in range(cmp::max(module.get_min_hp(), module.get_hp()), module.get_max_hp()) {
                armor_dmg_rect.draw([0.0, 4.0 * (i as f64), 8.0, 2.0], &context, gl);
            }
        }
    }
    
    #[cfg(feature = "client")]
    pub fn draw_module_powered_icons(&self, context: &Context, gl: &mut Gl, module_icons: &ModuleIcons) {
        use graphics::*;
    
        for module in self.modules.iter() {
            let module = module.borrow();
            let module = module.get_base();
            
            // Skip modules that aren't powerable
            if module.get_power() == 0 { continue; }
            
            // Skip modules that aren't changing powered states
            if module.plan_powered == module.powered { continue; }
            
            let context = context.trans((module.x as f64) * 48.0, (module.y as f64) * 48.0).trans((module.width as f64)*48.0 - 20.0, 2.0);
            
            if module.plan_powered {
                // Module is powering up, draw on icon
                image(&module_icons.power_on_texture, &context, gl);
            } else {
                // Module is powering down, draw off icon
                image(&module_icons.power_off_texture, &context, gl);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ShipStored {
    pub id: ShipId,
    pub name: String,
    pub state: ShipState,
    pub modules: Vec<ModuleStoredBox>,
    
    // Ship dimensions in module blocks
    width: u8,
    height: u8,
    
    pub level: u8, // TODO: This is very temporary only for IC US semifinals
}

impl ShipStored {
    pub fn new(id: ShipId, level: u8) -> ShipStored {
        ShipStored {
            id: id,
            name: String::new(),
            state: ShipState::new(),
            modules: vec!(),
            
            width: 0,
            height: 0,
            
            level: level,
        }
    }
    
    pub fn from_ship(ship: Ship) -> ShipStored {
        use std::rc::try_unwrap;
    
        ShipStored {
            id: ship.id,
            name: ship.name,
            state: ship.state,
            modules: ship.modules.into_iter().map(|m| try_unwrap(m).ok().expect("Failed to unwrap Module").into_inner().to_module_stored()).collect(),
            width: ship.width,
            height: ship.height,
            level: ship.level,
        }
    }
    
    pub fn to_ship(self, client_id: Option<ClientId>) -> Ship {
        Ship {
            id: self.id,
            name: self.name,
            client_id: client_id,
            state: self.state,
            modules: self.modules.into_iter().map(|m| Rc::new(RefCell::new(m.to_module()))).collect(),
            width: self.width,
            height: self.height,
            level: self.level,
        }
    }
}
