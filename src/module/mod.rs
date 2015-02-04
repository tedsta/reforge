use std::rc::Rc;
use std::cell::RefCell;
use std::any::TypeId;
use std::ops::{Deref, DerefMut};

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
    
    fn get_target_mode(&self, base: &ModuleBase) -> Option<TargetMode>;
    fn inject_target_data(&mut self, base: &mut ModuleBase, target_data: TargetData);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq)]
pub enum TargetMode {
    TargetShip,
    TargetModule,
    OwnModule,
    AnyModule,
    Beam,
}

pub enum TargetData {
    TargetShip(ShipRef),
    TargetModule(ShipRef, ModuleRef),
    OwnModule(ShipRef, ModuleRef),
    AnyModule(ShipRef, ModuleRef),
    Beam,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleBox(Box<IModuleRef + 'static>);
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
    
    fn to_module_stored(&self) -> ModuleStoredBox;
    
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
    
    fn get_target_mode(&self) -> Option<TargetMode>;
    fn inject_target_data(&mut self, target_data: TargetData);
}

impl<M> IModuleRef for Module<M>
    where M: IModule + Clone + 'static
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
    
    fn to_module_stored(&self) -> ModuleStoredBox {
        let base = ModuleBaseStored::from_module_base(&self.base);
    
        ModuleStoredBox::new(ModuleStored{base: base, module: self.module.clone()})
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
    
    fn get_target_mode(&self) -> Option<TargetMode> {
        self.module.get_target_mode(&self.base)
    }
    
    fn inject_target_data(&mut self, target_data: TargetData) {
        self.module.inject_target_data(&mut self.base, target_data);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*macro_rules! module_type_switch {
    ($)
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleStoredBox(Box<IModuleStored + 'static>);

pub struct ModuleStored<M: IModule> {
    base: ModuleBaseStored,
    module: M,
}

pub trait IModuleStored {
    fn to_module(&self) -> ModuleBox;
}

impl ModuleStoredBox {
    pub fn new<M>(module: M) -> ModuleStoredBox
        where M: IModuleStored + 'static
    {
        ModuleStoredBox(Box::new(module))
    }
}

/*impl ModuleStoredBox {
    fn from_module(module_box: ModuleBox) -> ModuleStoredBox {
        let type_id = module_box.get_type_id();
    
        let base = ModuleBaseStored::from_module_base(module_box.get_base());
    
        unsafe {
            if type_id == TypeId::of::<ProjectileWeaponModule>() {
                ModuleStoredBox(Box::new(ModuleStored{base: base, module: module_box.unpack_module::<ProjectileWeaponModule>().clone()}))
            } else if type_id == TypeId::of::<ShieldModule>() {
                ModuleStoredBox(Box::new(ModuleStored{base: base, module: module_box.unpack_module::<ShieldModule>().clone()}))
            } else if type_id == TypeId::of::<EngineModule>() {
                ModuleStoredBox(Box::new(ModuleStored{base: base, module: module_box.unpack_module::<EngineModule>().clone()}))
            } else if type_id == TypeId::of::<SolarModule>() {
                ModuleStoredBox(Box::new(ModuleStored{base: base, module: module_box.unpack_module::<SolarModule>().clone()}))
            } else if type_id == TypeId::of::<CommandModule>() {
                ModuleStoredBox(Box::new(ModuleStored{base: base, module: module_box.unpack_module::<CommandModule>().clone()}))
            } else { unreachable!() }
        }
    }
}*/

impl<M> IModuleStored for ModuleStored<M>
    where M: IModule+Clone + 'static
{   
    fn to_module(&self) -> ModuleBox {
        let base = self.base.to_module_base();
    
        ModuleBox::new(Module{base: base, module: self.module.clone()})
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(RustcEncodable, RustcDecodable)]
enum ModuleClass {
    ProjectileWeapon,
    Shield,
    Engine,
    Solar,
    Command,
}

// Some downcasting helper methods
impl ModuleBox {
    pub fn new<M>(module: M) -> ModuleBox
        where M: IModuleRef + 'static
    {
        ModuleBox(Box::new(module))
    }

    unsafe fn unpack_module<M: IModule>(&self) -> &M {
        use std::mem;
        use std::raw;
        
        // Get the &IModule trait object
        let module_to: raw::TraitObject = mem::transmute(self.get_module());
        
        // Get underlying module from IModule trait object
        let module: &M = mem::transmute(module_to.data);

        module
    }
}

impl Deref for ModuleBox {
    type Target = IModuleRef+'static;

    fn deref<'a>(&'a self) -> &'a (IModuleRef+'static) {
        let &ModuleBox(ref module_box) = self;
        module_box.deref()
    }
}

impl DerefMut for ModuleBox {
    fn deref_mut<'a>(&'a mut self) -> &'a mut (IModuleRef+'static) {
        let &mut ModuleBox(ref mut module_box) = self;
        module_box.deref_mut()
    }
}

impl Decodable for ModuleBox {
    fn decode<D: Decoder>(d: &mut D) -> Result<ModuleBox, D::Error> {
        use self::ModuleClass::*;
        
        let module_class: ModuleClass = try!(Decodable::decode(d));
        let base: ModuleBase = try!(Decodable::decode(d));
        
        match module_class {
            ProjectileWeapon => Ok(ModuleBox::new(Module {
                base: base,
                module: try!(<ProjectileWeaponModule as Decodable>::decode(d)),
            })),
            Shield => Ok(ModuleBox::new(Module {
                base: base,
                module: try!(<ShieldModule as Decodable>::decode(d)),
            })),
            Engine => Ok(ModuleBox::new(Module {
                base: base,
                module: try!(<EngineModule as Decodable>::decode(d)),
            })),
            Solar => Ok(ModuleBox::new(Module {
                base: base,
                module: try!(<SolarModule as Decodable>::decode(d)),
            })),
            Command => Ok(ModuleBox::new(Module {
                base: base,
                module: try!(<CommandModule as Decodable>::decode(d)),
            })),
        }
    }
}

impl Encodable for ModuleBox {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        use std::mem;
        use std::raw;
        
        use self::ModuleClass::*;
        
        let type_id = self.get_type_id();
        
        let module_class =
            if type_id == TypeId::of::<ProjectileWeaponModule>() { ProjectileWeapon }
            else if type_id == TypeId::of::<ShieldModule>() { Shield }
            else if type_id == TypeId::of::<EngineModule>() { Engine }
            else if type_id == TypeId::of::<SolarModule>() { Solar }
            else if type_id == TypeId::of::<CommandModule>() { Command }
            else { unreachable!() };
    
        try!(module_class.encode(s));
        try!(self.get_base().encode(s));
        
        match module_class {
            ProjectileWeapon => unsafe {
                let to: raw::TraitObject = mem::transmute(self.get_module());
                try!(<ProjectileWeaponModule as Encodable>::encode(mem::transmute(to.data), s));
            },
            Shield => unsafe {
                let to: raw::TraitObject = mem::transmute(self.get_module());
                try!(<ShieldModule as Encodable>::encode(mem::transmute(to.data), s));
            },
            Engine => unsafe {
                let to: raw::TraitObject = mem::transmute(self.get_module());
                try!(<EngineModule as Encodable>::encode(mem::transmute(to.data), s));
            },
            Solar => unsafe {
                let to: raw::TraitObject = mem::transmute(self.get_module());
                try!(<SolarModule as Encodable>::encode(mem::transmute(to.data), s));
            },
            Command => unsafe {
                let to: raw::TraitObject = mem::transmute(self.get_module());
                try!(<CommandModule as Encodable>::encode(mem::transmute(to.data), s));
            },
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct ModuleBase {
    // Module position/size stuff
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,

    // Module stats
    power: u8,     // Power consumption
    hp: u8,        // Total current HP of module, including armor
    min_hp: u8,    // Minimum HP for the module to still operate
    max_hp: u8,    // Maximum HP of module, including armor
    
    pub powered: bool,      // If the module consumes power, whether or not it's currently powered (useless otherwise)
    pub plan_powered: bool, // Plan to power
    
    pub index: u32, // Array index in ship. Used for referencing modules across network.
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

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleBaseStored {
    // Module position/size stuff
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,

    // Module stats
    power: u8,     // Power consumption
    hp: u8,        // Total current HP of module, including armor
    min_hp: u8,    // Minimum HP for the module to still operate
    max_hp: u8,    // Maximum HP of module, including armor
    
    pub powered: bool,      // If the module consumes power, whether or not it's currently powered (useless otherwise)
    
    pub index: u32, // Array index in ship. Used for referencing modules across network.
}

impl ModuleBaseStored {
    pub fn from_module_base(module_base: &ModuleBase) -> ModuleBaseStored {
        ModuleBaseStored {
            x: module_base.x,
            y: module_base.y,
            width: module_base.width,
            height: module_base.height,
            
            power: module_base.power,
            hp: module_base.hp,
            min_hp: module_base.min_hp,
            max_hp: module_base.max_hp,
            
            powered: module_base.powered,
            
            index: module_base.index,
        }
    }
    
    pub fn to_module_base(&self) -> ModuleBase {
        ModuleBase {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            
            power: self.power,
            hp: self.hp,
            min_hp: self.min_hp,
            max_hp: self.max_hp,
            
            powered: self.powered,
            plan_powered: self.powered,
            
            index: self.index,
        }
    }
}