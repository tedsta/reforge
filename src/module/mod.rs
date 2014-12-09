use std::rc::Rc;
use std::cell::RefCell;

use assets::TextureId;
use battle_state::BattleContext;
use net::{InPacket, OutPacket};
use ship::{ShipRef, ShipState};
use sim::{SimEventAdder, SimEvents};
use vec::{Vec2, Vec2f};

#[cfg(client)]
use sim::SimVisuals;
#[cfg(client)]
use asset_store::AssetStore;

// Use+reexport all of the modules
pub use self::engine::EngineModule;
pub use self::proj_weapon::ProjectileWeaponModule;
pub use self::shield::ShieldModule;
pub use self::solar::SolarModule;
pub use self::command::CommandModule;
pub use self::mod_type::{ModuleType, ModuleTypeInfo, ModuleTypeStore};

pub mod engine;
pub mod proj_weapon;
pub mod shield;
pub mod solar;
pub mod command;
pub mod mod_type;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Encodable, Decodable, PartialEq)]
pub enum ModuleCategory {
    Weapon = 0,
    Propulsion,
    Defense,
    Power,
    Control,
}

pub struct ModuleCategoryData {
    pub name: &'static str,
    pub id: ModuleCategory,
}

pub static MODULE_CATEGORIES: [ModuleCategoryData, .. 5] = [
    ModuleCategoryData{name: "Weapon", id: Weapon},
    ModuleCategoryData{name: "Propulsion", id: Propulsion},
    ModuleCategoryData{name: "Defense", id: Defense},
    ModuleCategoryData{name: "Power", id: Power},
    ModuleCategoryData{name: "Control", id: Control},
];

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IModule {
    fn server_preprocess(&mut self, ship_state: &mut ShipState);

    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder);
    #[cfg(client)]
    fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef);
    #[cfg(client)]
    fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef);
    fn after_simulation(&mut self, ship_state: &mut ShipState);

    fn write_plans(&self, packet: &mut OutPacket);
    fn read_plans(&mut self, context: &BattleContext, packet: &mut InPacket);
    
    fn write_results(&self, packet: &mut OutPacket);
    fn read_results(&mut self, packet: &mut InPacket);
    
    fn on_activated(&mut self, ship_state: &mut ShipState);
    fn on_deactivated(&mut self, ship_state: &mut ShipState);
    
    ////////////////////
    // GUI stuff
    
    fn on_icon_clicked(&mut self) -> bool;
    fn on_module_clicked(&mut self, ship: &ShipRef, module: &ModuleRef) -> bool;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleRef = Rc<RefCell<Module>>;

#[deriving(Encodable, Decodable)]
pub enum Module {
    Engine(EngineModule),
    ProjectileWeapon(ProjectileWeaponModule),
    Shield(ShieldModule),
    Solar(SolarModule),
    Command(CommandModule),
}

impl Module {
    pub fn get_base<'a>(&'a self) -> &'a ModuleBase {
        match (*self) {
            Engine(ref m) => &m.base,
            ProjectileWeapon(ref m) => &m.base,
            Shield(ref m) => &m.base,
            Solar(ref m) => &m.base,
            Command(ref m) => &m.base,
        }
    }
    
    pub fn get_base_mut<'a>(&'a mut self) -> &'a mut ModuleBase {
        match (*self) {
            Engine(ref mut m) => &mut m.base,
            ProjectileWeapon(ref mut m) => &mut m.base,
            Shield(ref mut m) => &mut m.base,
            Solar(ref mut m) => &mut m.base,
            Command(ref mut m) => &mut m.base,
        }
    }
}

impl IModule for Module {
    fn server_preprocess(&mut self, ship_state: &mut ShipState) {
        match *self {
            Engine(ref mut m) => m.server_preprocess(ship_state),
            ProjectileWeapon(ref mut m) => m.server_preprocess(ship_state),
            Shield(ref mut m) => m.server_preprocess(ship_state),
            Solar(ref mut m) => m.server_preprocess(ship_state),
            Command(ref mut m) => m.server_preprocess(ship_state),
        }
    }
    
    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder) {
        match *self {
            Engine(ref mut m) => m.before_simulation(ship_state, events),
            ProjectileWeapon(ref mut m) => m.before_simulation(ship_state, events),
            Shield(ref mut m) => m.before_simulation(ship_state, events),
            Solar(ref mut m) => m.before_simulation(ship_state, events),
            Command(ref mut m) => m.before_simulation(ship_state, events),
        }
    }
    
    #[cfg(client)]
    fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        match *self {
            Engine(ref m) => m.add_plan_visuals(asset_store, visuals, ship),
            ProjectileWeapon(ref m) => m.add_plan_visuals(asset_store, visuals, ship),
            Shield(ref m) => m.add_plan_visuals(asset_store, visuals, ship),
            Solar(ref m) => m.add_plan_visuals(asset_store, visuals, ship),
            Command(ref m) => m.add_plan_visuals(asset_store, visuals, ship),
        }
    }
    
    #[cfg(client)]
    fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        match *self {
            Engine(ref m) => m.add_simulation_visuals(asset_store, visuals, ship),
            ProjectileWeapon(ref m) => m.add_simulation_visuals(asset_store, visuals, ship),
            Shield(ref m) => m.add_simulation_visuals(asset_store, visuals, ship),
            Solar(ref m) => m.add_simulation_visuals(asset_store, visuals, ship),
            Command(ref m) => m.add_simulation_visuals(asset_store, visuals, ship),
        }
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
        match *self {
            Engine(ref mut m) => m.after_simulation(ship_state),
            ProjectileWeapon(ref mut m) => m.after_simulation(ship_state),
            Shield(ref mut m) => m.after_simulation(ship_state),
            Solar(ref mut m) => m.after_simulation(ship_state),
            Command(ref mut m) => m.after_simulation(ship_state),
        }
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
        match *self {
            Engine(ref m) => m.write_plans(packet),
            ProjectileWeapon(ref m) => m.write_plans(packet),
            Shield(ref m) => m.write_plans(packet),
            Solar(ref m) => m.write_plans(packet),
            Command(ref m) => m.write_plans(packet),
        }
    }
    
    fn read_plans(&mut self, context: &BattleContext, packet: &mut InPacket) {
        match *self {
            Engine(ref mut m) => m.read_plans(context, packet),
            ProjectileWeapon(ref mut m) => m.read_plans(context, packet),
            Shield(ref mut m) => m.read_plans(context, packet),
            Solar(ref mut m) => m.read_plans(context, packet),
            Command(ref mut m) => m.read_plans(context, packet),
        }
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
        match *self {
            Engine(ref m) => m.write_results(packet),
            ProjectileWeapon(ref m) => m.write_results(packet),
            Shield(ref m) => m.write_results(packet),
            Solar(ref m) => m.write_results(packet),
            Command(ref m) => m.write_results(packet),
        }
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
        match *self {
            Engine(ref mut m) => m.read_results(packet),
            ProjectileWeapon(ref mut m) => m.read_results(packet),
            Shield(ref mut m) => m.read_results(packet),
            Solar(ref mut m) => m.read_results(packet),
            Command(ref mut m) => m.read_results(packet),
        }
    }
    
    fn on_activated(&mut self, ship_state: &mut ShipState) {
        match *self {
            Engine(ref mut m) => m.on_activated(ship_state),
            ProjectileWeapon(ref mut m) => m.on_activated(ship_state),
            Shield(ref mut m) => m.on_activated(ship_state),
            Solar(ref mut m) => m.on_activated(ship_state),
            Command(ref mut m) => m.on_activated(ship_state),
        }
    }
    
    fn on_deactivated(&mut self, ship_state: &mut ShipState) {
        match *self {
            Engine(ref mut m) => m.on_deactivated(ship_state),
            ProjectileWeapon(ref mut m) => m.on_deactivated(ship_state),
            Shield(ref mut m) => m.on_deactivated(ship_state),
            Solar(ref mut m) => m.on_deactivated(ship_state),
            Command(ref mut m) => m.on_deactivated(ship_state),
        }
    }
    
    fn on_icon_clicked(&mut self) -> bool {
        match *self {
            Engine(ref mut m) => m.on_icon_clicked(),
            ProjectileWeapon(ref mut m) => m.on_icon_clicked(),
            Shield(ref mut m) => m.on_icon_clicked(),
            Solar(ref mut m) => m.on_icon_clicked(),
            Command(ref mut m) => m.on_icon_clicked(),
        }
    }
    
    fn on_module_clicked(&mut self, ship: &ShipRef, module: &ModuleRef) -> bool {
        match *self {
            Engine(ref mut m) => m.on_module_clicked(ship, module),
            ProjectileWeapon(ref mut m) => m.on_module_clicked(ship, module),
            Shield(ref mut m) => m.on_module_clicked(ship, module),
            Solar(ref mut m) => m.on_module_clicked(ship, module),
            Command(ref mut m) => m.on_module_clicked(ship, module),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Encodable, Decodable)]
pub struct ModuleBase {
    // Module position/size stuff
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,

    // Module stats
    power: u8,     // power consumption
    hp: u8,        // total current HP of module, including armor
    min_hp: u8,    // minimum HP for the module to still operate
    max_hp: u8,    // maximum HP of module, including armor
    
    pub powered: bool, // if the module consumes power, whether or not it's currently powered (useless otherwise)
    pub plan_powered: bool, // plan to power
    
    // Array index in ship. Used for referencing modules across network.
    pub index: u32,
    
    // Module type
    pub mod_type: ModuleType,
    
    // Category of this module
    pub category: ModuleCategory,
}

impl ModuleBase {
    pub fn new(mod_store: &ModuleTypeStore, mod_type: ModuleType, power: u8, min_hp: u8, hp: u8) -> ModuleBase {
        ModuleBase {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            
            power: power,
            hp: hp,
            min_hp: min_hp,
            max_hp: hp,
            
            powered: false,
            plan_powered: false,
            
            index: -1,
            
            mod_type: mod_type,
            category: mod_store.get_module_type(mod_type).category,
        }
    }
    
    pub fn get_power(&self) -> u8 {
        self.power
    }
    
    pub fn get_hp(&self) -> u8 {
        self.hp
    }
    
    pub fn get_min_hp(&self) -> u8 {
        self.min_hp
    }
    
    pub fn get_max_hp(&self) -> u8 {
        self.max_hp
    }
    
    pub fn can_activate(&self) -> bool {
        self.power > 0 && !self.powered && self.hp >= self.min_hp
    }
    
    pub fn is_active(&self) -> bool {
        self.hp >= self.min_hp && (self.powered || self.power == 0)
    }
    
    // Returns the amount of damage dealt
    pub fn deal_damage(&mut self, damage: u8) -> u8 {
        if self.hp >= damage {
            self.hp -= damage;
            damage
        } else {
            let dealt_damage = self.hp;
            self.hp = 0;
            dealt_damage
        }
    }
    
    pub fn get_render_position(&self) -> Vec2f {
        Vec2{x: (self.x as f64)*(48f64), y: (self.y as f64)*(48f64)}
    }
    
    pub fn get_render_size(&self) -> Vec2f {
        Vec2{x: (self.width as f64)*(48f64), y: (self.height as f64)*(48f64)}
    }
    
    pub fn get_render_center(&self) -> Vec2f {
        self.get_render_position() + (self.get_render_size()/2.0)
    }
}