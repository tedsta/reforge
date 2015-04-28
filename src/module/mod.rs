use std::any::TypeId;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rand;
use std::rand::Rng;
use std::marker::Reflect;

use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use battle_context::BattleContext;
use net::{InPacket, OutPacket};
use ship::{Ship, ShipId, ShipIndex, ShipState};
use sim::SimEvents;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use asset_store::AssetStore;

// Use+reexport all of the modules
pub use self::engine::EngineModule;
pub use self::proj_weapon::ProjectileWeaponModule;
pub use self::shield::ShieldModule;
pub use self::solar::SolarModule;
pub use self::command::CommandModule;
pub use self::beam_weapon::BeamWeaponModule;

pub use self::target::{Target, TargetMode, TargetData, TargetManifest, TargetManifestData};
pub use self::damage_visual::{DamageVisual, DamageVisualKind};

pub mod engine;
pub mod proj_weapon;
pub mod shield;
pub mod solar;
pub mod command;
pub mod beam_weapon;

pub mod target;
pub mod damage_visual;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IModule : Send {
    fn server_preprocess(&mut self, base: &mut ModuleBase, ship_state: &ShipState, target: Option<TargetManifest>) {}

    fn before_simulation(&mut self, base: &mut ModuleBase, events: &mut SimEvents, target: Option<TargetManifest>) {}
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, base: &ModuleBase, asset_store: &AssetStore, effects: &mut SimEffects, ship: &Ship);
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, base: &ModuleBase, asset_store: &AssetStore, effects: &mut SimEffects, ship: &Ship, target: Option<TargetManifest>);
    
    fn after_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {}
    
    fn write_results(&self, base: &ModuleBase, packet: &mut OutPacket) {}
    fn read_results(&mut self, base: &mut ModuleBase, packet: &mut InPacket) {}
    
    fn on_activated(&mut self, ship_state: &mut ShipState) {}
    fn on_deactivated(&mut self, ship_state: &mut ShipState) {}
    
    ////////////////////
    // GUI stuff
    
    fn get_target_mode(&self, base: &ModuleBase) -> Option<TargetMode>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleBox(Box<IModuleRef + 'static>);
pub type ModuleRef = Rc<RefCell<ModuleBox>>;

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
    
    fn server_preprocess(&mut self, ship_state: &ShipState, target: Option<TargetManifest>);

    fn before_simulation(&mut self, events: &mut SimEvents, target: Option<TargetManifest>);
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects, ship: &Ship);
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects, ship: &Ship, target: Option<TargetManifest>);
    fn after_simulation(&mut self, ship_state: &mut ShipState);
    
    fn write_results(&self, packet: &mut OutPacket);
    fn read_results(&mut self, packet: &mut InPacket);
    
    fn on_activated(&mut self, ship_state: &mut ShipState);
    fn on_deactivated(&mut self, ship_state: &mut ShipState);
    
    ////////////////////
    // GUI stuff
    
    fn get_target_mode(&self) -> Option<TargetMode>;
}

impl<M> IModuleRef for Module<M>
    where M: IModule + Reflect + Clone + 'static
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
    
    fn server_preprocess(&mut self, ship_state: &ShipState, target: Option<TargetManifest>) {
        self.module.server_preprocess(&mut self.base, ship_state, target);
    }
    
    fn before_simulation(&mut self, events: &mut SimEvents, target: Option<TargetManifest>) {
        self.module.before_simulation(&mut self.base, events, target);
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects, ship: &Ship) {
        self.module.add_plan_effects(&self.base, asset_store, effects, ship);
        self.base.add_damage_effects(asset_store, effects, ship.id);
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects, ship: &Ship, target: Option<TargetManifest>) {
        self.module.add_simulation_effects(&self.base, asset_store, effects, ship, target);
        self.base.add_damage_effects(asset_store, effects, ship.id);
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
        self.module.after_simulation(&mut self.base, ship_state);
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
        self.module.write_results(&self.base, packet);
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
        self.module.read_results(&mut self.base, packet);
    }
    
    fn on_activated(&mut self, ship_state: &mut ShipState) {
        self.module.on_activated(ship_state);
    }
    
    fn on_deactivated(&mut self, ship_state: &mut ShipState) {
        self.module.on_deactivated(ship_state);
    }
    
    fn get_target_mode(&self) -> Option<TargetMode> {
        self.module.get_target_mode(&self.base)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleStoredBox(Box<IModuleStored + 'static>);

pub struct ModuleStored<M: IModule> {
    base: ModuleBaseStored,
    module: M,
}

pub trait IModuleStored : Send {
    fn to_module(&self) -> ModuleBox;
}

impl ModuleStoredBox {
    pub fn new<M>(module: M) -> ModuleStoredBox
        where M: IModuleStored + 'static
    {
        ModuleStoredBox(Box::new(module))
    }
}

impl<M> IModuleStored for ModuleStored<M>
    where M: IModule+Reflect+Clone + 'static
{   
    fn to_module(&self) -> ModuleBox {
        let base = self.base.to_module_base();
    
        ModuleBox::new(Module{base: base, module: self.module.clone()})
    }
}

impl Deref for ModuleStoredBox {
    type Target = IModuleStored+'static;

    fn deref<'a>(&'a self) -> &'a (IModuleStored+'static) {
        let &ModuleStoredBox(ref module_stored_box) = self;
        module_stored_box.deref()
    }
}

impl DerefMut for ModuleStoredBox {
    fn deref_mut<'a>(&'a mut self) -> &'a mut (IModuleStored+'static) {
        let &mut ModuleStoredBox(ref mut module_stored_box) = self;
        module_stored_box.deref_mut()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Some downcasting helper methods
impl ModuleBox {
    pub fn new<M>(module: M) -> ModuleBox
        where M: IModuleRef + 'static
    {
        ModuleBox(Box::new(module))
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Serialization

#[derive(RustcEncodable, RustcDecodable)]
enum ModuleClass {
    ProjectileWeapon,
    Shield,
    Engine,
    Solar,
    Command,
    BeamWeapon,
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
            BeamWeapon => Ok(ModuleBox::new(Module {
                base: base,
                module: try!(<BeamWeaponModule as Decodable>::decode(d)),
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
            else if type_id == TypeId::of::<BeamWeaponModule>() { BeamWeapon }
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
            BeamWeapon => unsafe {
                let to: raw::TraitObject = mem::transmute(self.get_module());
                try!(<BeamWeaponModule as Encodable>::encode(mem::transmute(to.data), s));
            },
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(RustcEncodable, RustcDecodable)]
pub struct ModulePlans {
    pub plan_powered: bool,
    pub plan_target: Option<Target>,
}

impl ModulePlans {
    pub fn on_ship_removed(&mut self, ship: ShipIndex) {
        use self::TargetData::*;
    
        // TODO make this prettier
        
        let mut remove = false;
    
        if let Some(ref target) = self.plan_target {
            if target.ship == ship {
                remove = true;
            }
        }
        
        if remove {
            self.plan_target = None;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct ModuleStats {
    pub hp: u8,
}

impl ModuleStats {
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct ModuleIndex(pub u32);

impl ModuleIndex {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct ModuleBase {
    // Module position/size stuff
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,

    pub stats: ModuleStats,
    
    power: u8,     // Power consumption
    min_hp: u8,    // Minimum HP for the module to still operate
    max_hp: u8,    // Maximum HP of module, including armor
    
    pub powered: bool, // If the module consumes power, whether or not it's currently powered
    
    pub target: Option<Target>,
    
    // Module damage visuals
    damage_visuals: Vec<DamageVisual>,
    
    pub index: ModuleIndex, // Array index in ship. Used for referencing modules.
}

impl ModuleBase {
    pub fn new(width: u8, height: u8, power: u8, min_hp: u8, hp: u8) -> ModuleBase {
        ModuleBase {
            x: 0,
            y: 0,
            width: width,
            height: height,
            
            stats: ModuleStats { hp: hp },
            
            power: power,
            min_hp: min_hp,
            max_hp: hp,
            
            powered: false,
            
            target: None,
            
            damage_visuals: vec!(),
            
            index: ModuleIndex(-1),
        }
    }
    
    pub fn get_power(&self) -> u8 {
        self.power
    }
    
    pub fn get_hp(&self) -> u8 {
        self.stats.hp
    }
    
    pub fn get_min_hp(&self) -> u8 {
        self.min_hp
    }
    
    pub fn get_max_hp(&self) -> u8 {
        self.max_hp
    }
    
    pub fn can_activate(&self) -> bool {
        self.power > 0 && self.stats.hp >= self.min_hp
    }
    
    pub fn is_active(&self) -> bool {
        self.stats.hp >= self.min_hp && (self.powered || self.power == 0)
    }
    
    // Returns the amount of damage dealt
    pub fn deal_damage(&mut self, damage: u8) -> u8 {
        let dealt_damage = self.stats.deal_damage(damage);
        
        // Create damage visual at random location
        if self.stats.hp < self.min_hp {
            // Random number generater
            let mut rng = rand::thread_rng();
            
            let x = rng.gen::<f64>() * ((self.width as f64) * 48.0);
            let y = rng.gen::<f64>() * ((self.height as f64) * 48.0);

            self.damage_visuals.push(DamageVisual {
                x: x,
                y: y,
                kind: DamageVisualKind::Fire,
            });
        }
        
        dealt_damage
    }
    
    #[cfg(feature = "client")]
    pub fn add_damage_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects, ship_id: ShipId) {
        use sim_visuals::SpriteVisual;
        use sprite_sheet::{SpriteSheet, SpriteAnimation};
    
        for visual in &self.damage_visuals {
            let mut sprite = SpriteSheet::new(asset_store.get_sprite_info_str("effects/fire_sprite.png"));
            sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 7, 0.05));
        
            effects.add_visual(ship_id, 1, box SpriteVisual {
                position: self.get_render_position().clone() + Vec2 { x: 10.0, y: 0.0 },
                sprite_sheet: sprite,
            });
        }
    }
    
    pub fn create_plans(&self) -> ModulePlans {
        ModulePlans {
            plan_powered: self.powered,
            plan_target: self.target,
        }
    }
    
    pub fn on_ship_removed(&mut self, ship: ShipIndex) {
        use self::TargetData::*;
    
        // TODO make this prettier
        
        let mut remove = false;
        
        if let Some(ref target) = self.target {
            if target.ship == ship {
                remove = true;
            }
        }
        
        if remove {
            self.target = None;
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

    stats: ModuleStats,
    
    power: u8,     // Power consumption
    min_hp: u8,    // Minimum HP for the module to still operate
    max_hp: u8,    // Maximum HP of module, including armor
    
    pub powered: bool,      // If the module consumes power, whether or not it's currently powered (useless otherwise)
    
    pub index: ModuleIndex, // Array index in ship. Used for referencing modules across network.
}

impl ModuleBaseStored {
    pub fn from_module_base(module_base: &ModuleBase) -> ModuleBaseStored {
        ModuleBaseStored {
            x: module_base.x,
            y: module_base.y,
            width: module_base.width,
            height: module_base.height,
            
            stats: module_base.stats,
            
            power: module_base.power,
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
            
            stats: self.stats,
            
            power: self.power,
            min_hp: self.min_hp,
            max_hp: self.max_hp,
            
            powered: self.powered,
            
            target: None,
            
            damage_visuals: vec!(),
            
            index: self.index,
        }
    }
}
