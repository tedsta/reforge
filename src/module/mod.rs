use std::rc::Rc;
use std::cell::RefCell;
use std::intrinsics::TypeId;

use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use assets::TextureId;
use battle_state::BattleContext;
use net::{InPacket, OutPacket};
use ship::{ShipId, ShipRef, ShipState};
use sim::{SimEventAdder, SimEvents};
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim::SimVisuals;
#[cfg(feature = "client")]
use asset_store::AssetStore;

// Use+reexport all of the modules
pub use self::engine::EngineModule;
pub use self::proj_weapon::ProjectileWeaponModule;
pub use self::shield::ShieldModule;
pub use self::solar::SolarModule;
pub use self::command::CommandModule;

pub mod engine;
pub mod proj_weapon;
pub mod shield;
pub mod solar;
pub mod command;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IModule {
    fn server_preprocess(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState);

    fn before_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, events: &mut SimEventAdder);
    #[cfg(feature = "client")]
    fn add_plan_visuals(&self, base: &ModuleBase, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef);
    #[cfg(feature = "client")]
    fn add_simulation_visuals(&self, base: &ModuleBase, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef);
    fn after_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState);
    
    fn on_ship_removed(&mut self, base: &mut ModuleBase, _: ShipId) {}

    fn write_plans(&self, base: &ModuleBase, packet: &mut OutPacket);
    fn read_plans(&mut self, base: &mut ModuleBase, context: &BattleContext, packet: &mut InPacket);
    
    fn write_results(&self, base: &ModuleBase, packet: &mut OutPacket);
    fn read_results(&mut self, base: &mut ModuleBase, packet: &mut InPacket);
    
    fn on_activated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>);
    fn on_deactivated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>);
    
    ////////////////////
    // GUI stuff
    
    fn on_icon_clicked(&mut self, base: &mut ModuleBase) -> bool;
    fn on_module_clicked(&mut self, base: &mut ModuleBase, ship: &ShipRef, module: &ModuleRef) -> bool;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleBox = Box<IModuleRef + 'static>;
pub type ModuleRef = Rc<RefCell<ModuleBox>>;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Module<M: IModule> {
    base: ModuleBase,
    module: M,
}

pub trait IModuleRef {
    fn get_type_id(&self) -> TypeId;
    fn get_base(&self) -> &ModuleBase;
    fn get_base_mut(&mut self) -> &mut ModuleBase;
    fn get_module(&self) -> &IModule;
    
    //////////////////////////////////////////////////////
    // IModule stuff
    
    fn server_preprocess(&mut self, ship_state: &mut ShipState);

    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder);
    #[cfg(feature = "client")]
    fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef);
    #[cfg(feature = "client")]
    fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef);
    fn after_simulation(&mut self, ship_state: &mut ShipState);
    
    fn on_ship_removed(&mut self, ShipId) {}

    fn write_plans(&self, packet: &mut OutPacket);
    fn read_plans(&mut self, context: &BattleContext, packet: &mut InPacket);
    
    fn write_results(&self, packet: &mut OutPacket);
    fn read_results(&mut self, packet: &mut InPacket);
    
    fn on_activated(&mut self, ship_state: &mut ShipState, modules: &Vec<ModuleRef>);
    fn on_deactivated(&mut self, ship_state: &mut ShipState, modules: &Vec<ModuleRef>);
    
    ////////////////////
    // GUI stuff
    
    fn on_icon_clicked(&mut self) -> bool;
    fn on_module_clicked(&mut self, ship: &ShipRef, module: &ModuleRef) -> bool;
}

impl<M> IModuleRef for Module<M>
    where M: IModule + 'static
{
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<M>()
    }
    
    fn get_base(&self) -> &ModuleBase {
        &self.base
    }
    
    fn get_base_mut(&mut self) -> &mut ModuleBase {
        &mut self.base
    }
    
    fn get_module(&self) -> &IModule {
        &self.module
    }
    
    //////////////////////////////////////////////////////
    // IModule stuff
    
    fn server_preprocess(&mut self, ship_state: &mut ShipState) {
        self.module.server_preprocess(&mut self.base, ship_state);
    }
    
    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder) {
        self.module.before_simulation(&mut self.base, ship_state, events);
    }
    
    #[cfg(feature = "client")]
    fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        self.module.add_plan_visuals(&self.base, asset_store, visuals, ship);
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        self.module.add_simulation_visuals(&self.base, asset_store, visuals, ship);
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
        self.module.after_simulation(&mut self.base, ship_state);
    }
    
    fn on_ship_removed(&mut self, ship_id: ShipId) {
        self.module.on_ship_removed(&mut self.base, ship_id);
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
        self.module.write_plans(&self.base, packet);
    }
    
    fn read_plans(&mut self, context: &BattleContext, packet: &mut InPacket) {
        self.module.read_plans(&mut self.base, context, packet);
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
        self.module.write_results(&self.base, packet);
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
        self.module.read_results(&mut self.base, packet);
    }
    
    fn on_activated(&mut self, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
        self.module.on_activated(&mut self.base, ship_state, modules);
    }
    
    fn on_deactivated(&mut self, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
        self.module.on_deactivated(&mut self.base, ship_state, modules);
    }
    
    fn on_icon_clicked(&mut self) -> bool {
        self.module.on_icon_clicked(&mut self.base)
    }
    
    fn on_module_clicked(&mut self, ship: &ShipRef, module: &ModuleRef) -> bool {
        self.module.on_module_clicked(&mut self.base, ship, module)
    }
}

impl Decodable for ModuleBox {
    fn decode<D: Decoder>(d: &mut D) -> Result<ModuleBox, D::Error> {
        let module_class: TypeId = Decodable::decode(d).ok().expect("Failed to decode module_class");
        Ok(Box::new(Module {
            base: Decodable::decode(d).ok().expect("Failed to decode ModuleBase"),
            module: <ProjectileWeaponModule as Decodable>::decode(d).ok().expect("Failed to decode module"),
        }))
    }
}

impl Encodable for ModuleBox {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        use std::mem;
        use std::raw;
    
        self.get_base().encode(s).ok().expect("Failed to encode ModuleBase");
        unsafe {
            let to: raw::TraitObject = mem::transmute(self.get_module());
            <ProjectileWeaponModule as Encodable>::encode(mem::transmute(to.data), s).ok().expect("Failed to encode module");
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(RustcEncodable, RustcDecodable)]
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
}

impl ModuleBase {
    pub fn new(width: u8, height: u8, power: u8, min_hp: u8, hp: u8) -> ModuleBase {
        ModuleBase {
            x: 0,
            y: 0,
            width: width,
            height: height,
            
            power: power,
            hp: hp,
            min_hp: min_hp,
            max_hp: hp,
            
            powered: false,
            plan_powered: false,
            
            index: -1,
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
    
    pub fn can_plan_activate(&self) -> bool {
        self.power > 0 && !self.plan_powered && self.hp >= self.min_hp
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
        Vec2{x: (self.x as f64) * 48.0, y: (self.y as f64) * 48.0}
    }
    
    pub fn get_render_size(&self) -> Vec2f {
        Vec2{x: (self.width as f64) * 48.0, y: (self.height as f64) * 48.0}
    }
    
    pub fn get_render_center(&self) -> Vec2f {
        self.get_render_position() + (self.get_render_size()/2.0)
    }
}