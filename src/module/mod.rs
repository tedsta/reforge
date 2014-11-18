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
pub use self::mod_type::{ModuleType, ModuleTypeInfo, ModuleTypeStore};

pub mod engine;
pub mod proj_weapon;
pub mod shield;
pub mod mod_type;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Encodable, Decodable, PartialEq)]
pub enum ModuleCategory {
    Weapon = 0,
    Propulsion,
    Defense,
}

pub struct ModuleCategoryData {
    pub name: &'static str,
    pub id: ModuleCategory,
}

pub static MODULE_CATEGORIES: [ModuleCategoryData, .. 3] = [
    ModuleCategoryData{name: "Weapon", id: Weapon},
    ModuleCategoryData{name: "Propulsion", id: Propulsion},
    ModuleCategoryData{name: "Defense", id: Defense},
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
}

impl Module {
    pub fn get_base<'a>(&'a self) -> &'a ModuleBase {
        match (*self) {
            Engine(ref m) => &m.base,
            ProjectileWeapon(ref m) => &m.base,
            Shield(ref m) => &m.base,
        }
    }
    
    pub fn get_base_mut<'a>(&'a mut self) -> &'a mut ModuleBase {
        match (*self) {
            Engine(ref mut m) => &mut m.base,
            ProjectileWeapon(ref mut m) => &mut m.base,
            Shield(ref mut m) => &mut m.base,
        }
    }
}

impl IModule for Module {
    fn server_preprocess(&mut self, ship_state: &mut ShipState) {
        match *self {
            Engine(ref mut m) => m.server_preprocess(ship_state),
            ProjectileWeapon(ref mut m) => m.server_preprocess(ship_state),
            Shield(ref mut m) => m.server_preprocess(ship_state),
        }
    }
    
    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder) {
        match *self {
            Engine(ref mut m) => m.before_simulation(ship_state, events),
            ProjectileWeapon(ref mut m) => m.before_simulation(ship_state, events),
            Shield(ref mut m) => m.before_simulation(ship_state, events),
            }
    }
    
    #[cfg(client)]
    fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        match *self {
            Engine(ref m) => m.add_plan_visuals(asset_store, visuals, ship),
            ProjectileWeapon(ref m) => m.add_plan_visuals(asset_store, visuals, ship),
            Shield(ref m) => m.add_plan_visuals(asset_store, visuals, ship),
        }
    }
    
    #[cfg(client)]
    fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        match *self {
            Engine(ref m) => m.add_simulation_visuals(asset_store, visuals, ship),
            ProjectileWeapon(ref m) => m.add_simulation_visuals(asset_store, visuals, ship),
            Shield(ref m) => m.add_simulation_visuals(asset_store, visuals, ship),
        }
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
        match *self {
            Engine(ref mut m) => m.after_simulation(ship_state),
            ProjectileWeapon(ref mut m) => m.after_simulation(ship_state),
            Shield(ref mut m) => m.after_simulation(ship_state),
        }
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
        match *self {
            Engine(ref m) => m.write_plans(packet),
            ProjectileWeapon(ref m) => m.write_plans(packet),
            Shield(ref m) => m.write_plans(packet),
        }
    }
    
    fn read_plans(&mut self, context: &BattleContext, packet: &mut InPacket) {
        match *self {
            Engine(ref mut m) => m.read_plans(context, packet),
            ProjectileWeapon(ref mut m) => m.read_plans(context, packet),
            Shield(ref mut m) => m.read_plans(context, packet),
        }
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
        match *self {
            Engine(ref m) => m.write_results(packet),
            ProjectileWeapon(ref m) => m.write_results(packet),
            Shield(ref m) => m.write_results(packet),
        }
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
        match *self {
            Engine(ref mut m) => m.read_results(packet),
            ProjectileWeapon(ref mut m) => m.read_results(packet),
            Shield(ref mut m) => m.read_results(packet),
        }
    }
    
    fn on_icon_clicked(&mut self) -> bool {
        match *self {
            Engine(ref mut m) => m.on_icon_clicked(),
            ProjectileWeapon(ref mut m) => m.on_icon_clicked(),
            Shield(ref mut m) => m.on_icon_clicked(),
        }
    }
    
    fn on_module_clicked(&mut self, ship: &ShipRef, module: &ModuleRef) -> bool {
        match *self {
            Engine(ref mut m) => m.on_module_clicked(ship, module),
            ProjectileWeapon(ref mut m) => m.on_module_clicked(ship, module),
            Shield(ref mut m) => m.on_module_clicked(ship, module),
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
    power: u8,
    max_power: u8,
    hp: u8,
    max_hp: u8,
    
    // Module type
    pub mod_type: ModuleType,
    
    // Category of this module
    pub category: ModuleCategory,
}

impl ModuleBase {
    pub fn new(mod_store: &ModuleTypeStore, mod_type: ModuleType) -> ModuleBase {
        ModuleBase {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            power: 0,
            max_power: 1,
            hp: 0,
            max_hp: 0,
            mod_type: mod_type,
            category: mod_store.get_module_type(mod_type).category,
        }
    }
    
    pub fn get_power(&self) -> u8 {
        self.power
    }
    
    pub fn get_max_power(&self) -> u8 {
        self.max_power
    }
    
    pub fn get_hp(&self) -> u8 {
        self.hp
    }
    
    pub fn get_max_hp(&self) -> u8 {
        self.max_hp
    }
    
    pub fn deal_damage(&mut self, damage: u8) {
        if self.hp >= damage {
            self.hp -= damage;
        } else {
            self.hp = 0;
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