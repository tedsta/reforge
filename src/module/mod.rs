use std::any::TypeId;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use rand;
use rand::Rng;
//use std::marker::Reflect;

use serde::{Serialize, Serializer, Deserialize, Deserializer};

use battle_context::BattleContext;
use net::{InPacket, OutPacket};
use ship::{Ship, ShipId, ShipIndex, ShipState, ShipStored};
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
pub use self::cabin::CabinModule;
pub use self::command::CommandModule;
pub use self::beam_weapon::BeamWeaponModule;
pub use self::repair::RepairModule;
pub use self::volley_missile::VolleyMissileModule;

pub use self::target::{Target, TargetMode, TargetData, TargetManifest, TargetManifestData};
pub use self::damage_visual::{DamageVisual, DamageVisualKind};
pub use self::model::{Model, ModelIndex, ModelStore};

pub mod engine;
pub mod proj_weapon;
pub mod shield;
pub mod solar;
pub mod cabin;
pub mod command;
pub mod beam_weapon;
pub mod repair;
pub mod volley_missile;

pub mod target;
pub mod damage_visual;
pub mod model;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleContext<'a> {
    pub x: u8,
    pub y: u8,
    pub shape: &'a ModuleShape,
    pub model: &'a Model,

    pub index: ModuleIndex,
    pub is_active: bool,
    pub target: Option<TargetManifest<'a>>,
    
    pub ship_id: ShipId,
    pub ship_state: &'a ShipState,
    pub ship_position: Vec2f,
    pub ship_next_waypoint: Option<Vec2f>,
}

impl<'a> ModuleContext<'a> {
    pub fn get_render_position(&self) -> Vec2f {
        Vec2{x: (self.x as f64) * 48.0, y: (self.y as f64) * 48.0}
    }
    
    pub fn get_render_size(&self) -> Vec2f {
        Vec2{x: (self.shape.side() as f64) * 48.0, y: (self.shape.side() as f64) * 48.0}
    }
    
    pub fn get_render_center(&self) -> Vec2f {
        self.get_render_position() + (self.get_render_size()/2.0)
    }

    pub fn ship_lerp_next_waypoint(&self, time: f64) -> Vec2f {
        if let Some(next_waypoint) = self.ship_next_waypoint {
            self.ship_position + (next_waypoint - self.ship_position)*(time/5.0)
        } else {
            self.ship_position
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IModule : Send {
    fn get_class(&self) -> ModuleClass;
    fn get_target_mode(&self) -> Option<TargetMode> { None }

    fn server_preprocess(&mut self, context: &ModuleContext) {}

    fn before_simulation(&mut self, context: &ModuleContext, events: &mut SimEvents) {}
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects);
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects);
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {}
    
    fn write_results(&self, packet: &mut OutPacket) {}
    fn read_results(&mut self, packet: &mut InPacket) {}
    
    fn on_activated(&mut self, ship_state: &mut ShipState) {}
    fn on_deactivated(&mut self, ship_state: &mut ShipState) {}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Serialize, Deserialize)]
pub struct ModulePlans {
    pub active: bool,
    pub target: Option<Target>,
}

impl ModulePlans {
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ModuleStats {
    pub hp: u8,
    pub max_hp: u8,
}

impl ModuleStats {
    // Returns the amount of damage dealt
    pub fn deal_damage(&mut self, damage: u8) {
        if self.hp >= damage {
            self.hp -= damage;
        } else {
            self.hp = 0;
        }
    }

    pub fn repair_damage(&mut self, repair: u8) -> u8 {
        if self.max_hp - self.hp >= repair {
            self.hp += repair;
            repair
        } else {
            let dealt_repair = self.max_hp - self.hp;
            self.hp = self.max_hp;
            dealt_repair
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleIndex(pub u32);

impl ModuleIndex {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
    
    pub fn get<'a>(&self, ship: &'a Ship) -> &'a Module {
        &ship.modules[self.0 as usize]
    }
    
    pub fn get_mut<'a>(&self, ship: &'a mut Ship) -> &'a mut Module {
        &mut ship.modules[self.0 as usize]
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Serialize, Deserialize)]
pub struct ModuleShape {
    grid: Vec<Vec<u8>>,
    side: u8, // Has to be a square, because squares are easy to rotate
}

impl ModuleShape {
    pub fn new(grid: Vec<Vec<u8>>) -> ModuleShape {
        ModuleShape {
            side: grid.len() as u8,
            grid: grid,
        }
    }
    
    pub fn rotate_90(&mut self) {
        for n in (0..self.side - 1) {
            for m in (n + 1..self.side) {
                let tmp = self.get(m, n);
                *self.get_mut(m, n) = self.get(n, m);
                *self.get_mut(n, m) = tmp;
            }
        }
    }
    
    pub fn get(&self, x: u8, y: u8) -> u8 {
        *self.grid.get(y as usize).unwrap().get(x as usize).unwrap()
    }
    
    pub fn get_mut(&mut self, x: u8, y: u8) -> &mut u8 {
        self.grid.get_mut(y as usize).unwrap().get_mut(x as usize).unwrap()
    }
    
    pub fn side(&self) -> u8 {
        self.side
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct Module {
    pub model: ModelIndex,

    // Module position/size stuff
    pub x: u8,
    pub y: u8,
    pub shape: ModuleShape,

    pub stats: ModuleStats,
    
    power: u8,     // Power consumption
    min_hp: u8,    // Minimum HP for the module to still operate
    max_hp: u8,    // Maximum HP of module, including armor
    
    pub active: bool,
    
    pub target: Option<Target>,
    
    // Module damage visuals
    damage_visuals: Vec<DamageVisual>,
    
    pub index: ModuleIndex, // Array index in ship. Used for referencing modules.
    
    pub inner: RefCell<ModuleInnerBox>,
}

impl Module {
    pub fn new<M: IModule+'static>(
        model: ModelIndex,
        shape: ModuleShape,
        power: u8,
        min_hp: u8,
        hp: u8,
        inner: M,
    ) -> Module {
        Module {
            model: model,
            
            x: 0,
            y: 0,
            shape: shape,
            
            stats: ModuleStats { hp: hp, max_hp: hp },
            
            power: power,
            min_hp: min_hp,
            max_hp: hp,
            
            active: false,
            
            target: None,
            
            damage_visuals: vec!(),
            
            index: ModuleIndex(0),
            
            inner: RefCell::new(Box::new(inner)),
        }
    }
    
    pub fn from_model<M: IModule+'static>(model: &Model, inner: M) -> Module {
        Module {
            model: model.index,
        
            x: 0,
            y: 0,
            shape: model.shape.clone(),
            
            stats: ModuleStats { hp: model.max_hp, max_hp: model.max_hp },
            
            power: model.power,
            min_hp: model.min_hp,
            max_hp: model.max_hp,
            
            active: false,
            
            target: None,
            
            damage_visuals: vec!(),
            
            index: ModuleIndex(0),
            
            inner: RefCell::new(Box::new(inner)),
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
    
    pub fn get_class(&self) -> ModuleClass {
        self.inner.borrow().get_class()
    }
    
    pub fn get_target_mode(&self) -> Option<TargetMode> {
        self.inner.borrow().get_target_mode()
    }
    
    pub fn is_damaged(&self) -> bool {
        self.stats.hp < self.min_hp
    }
    
    pub fn can_activate(&self) -> bool {
        self.power > 0 && !self.is_damaged()
    }
    
    // Returns the amount of damage dealt
    pub fn deal_damage(&mut self, damage: u8) {
        self.stats.deal_damage(damage);
        
        // Create damage visual at random location
        if self.stats.hp < self.min_hp {
            // Random number generater
            let mut rng = rand::thread_rng();
            
            let x = rng.gen::<f64>() * ((self.shape.side() as f64) * 48.0);
            let y = rng.gen::<f64>() * ((self.shape.side() as f64) * 48.0);

            self.damage_visuals.push(DamageVisual {
                x: x,
                y: y,
                kind: DamageVisualKind::Fire,
            });
        }
    }
    
    // Returns the amount of damage repaired
    pub fn repair_damage(&mut self, repair: u8) -> u8 {
        if self.max_hp - self.stats.hp >= repair {
            self.stats.hp += repair;
            repair
        } else {
            let dealt_repair = self.max_hp - self.stats.hp;
            self.stats.hp = self.max_hp;
            dealt_repair
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_damage_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects, ship_id: ShipId) {
        use sim_visuals::SpriteVisual;
        use sprite_sheet::{SpriteSheet, SpriteAnimation};
    
        for visual in &self.damage_visuals {
            let mut sprite = SpriteSheet::new(asset_store.get_sprite_info_str("fire"));
            sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 7, 0.05));
        
            effects.add_visual(ship_id, 1,
                SpriteVisual::new(
                    self.get_render_position() + Vec2 { x: 10.0, y: 0.0 },
                    0.0,
                    sprite,
                ),
            );
        }
    }
    
    pub fn create_plans(&self) -> ModulePlans {
        ModulePlans {
            active: self.active,
            target: self.target,
        }
    }
    
    pub fn create_module_context<'a>(&'a self, bc: &'a BattleContext, model_store: &'a ModelStore, ship: &'a Ship) -> ModuleContext<'a> {
        ModuleContext {
            model: self.model.get(model_store),
            
            x: self.x,
            y: self.y,
            shape: &self.shape,
            
            index: self.index,
            is_active: self.active,
            target: self.target.as_ref().map(|t| TargetManifest::from_target(bc, t)),
            
            ship_id: ship.id,
            ship_state: &ship.state,
            ship_position: ship.position,
            ship_next_waypoint: ship.next_waypoint,
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
        Vec2{x: (self.shape.side() as f64) * 48.0, y: (self.shape.side() as f64) * 48.0}
    }
    
    pub fn get_render_center(&self) -> Vec2f {
        self.get_render_position() + (self.get_render_size()/2.0)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct ModuleStored {
    pub model: ModelIndex,

    // Module position/size stuff
    pub x: u8,
    pub y: u8,
    pub shape: ModuleShape,

    pub stats: ModuleStats,
    
    power: u8,     // Power consumption
    min_hp: u8,    // Minimum HP for the module to still operate
    max_hp: u8,    // Maximum HP of module, including armor
    
    pub active: bool,
    
    pub index: ModuleIndex, // Array index in ship. Used for referencing modules across network.
    
    pub inner: RefCell<ModuleInnerBox>,
}

impl ModuleStored {
    pub fn from_module(module: Module) -> ModuleStored {
        ModuleStored {
            model: module.model,
        
            x: module.x,
            y: module.y,
            shape: module.shape,
            
            stats: module.stats,
            
            power: module.power,
            min_hp: module.min_hp,
            max_hp: module.max_hp,
            
            active: module.active,
            
            index: module.index,
            
            inner: module.inner,
        }
    }
    
    pub fn to_module(self) -> Module {
        Module {
            model: self.model,
        
            x: self.x,
            y: self.y,
            shape: self.shape,
            
            stats: self.stats,
            
            power: self.power,
            min_hp: self.min_hp,
            max_hp: self.max_hp,
            
            active: self.active,
            
            target: None,
            
            damage_visuals: vec!(),
            
            index: self.index,
            
            inner: self.inner,
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
    
    pub fn get_class(&self) -> ModuleClass {
        self.inner.borrow().get_class()
    }
    
    pub fn get_target_mode(&self) -> Option<TargetMode> {
        self.inner.borrow().get_target_mode()
    }
    
    pub fn is_damaged(&self) -> bool {
        self.stats.hp < self.min_hp
    }
    
    pub fn can_activate(&self) -> bool {
        self.power > 0 && !self.is_damaged()
    }
    
    pub fn create_module_context<'a>(&'a self, model_store: &'a ModelStore, ship: &'a ShipStored) -> ModuleContext<'a> {
        ModuleContext {
            model: self.model.get(model_store),
        
            x: self.x,
            y: self.y,
            shape: &self.shape,
            
            index: self.index,
            is_active: self.active,
            target: None,
            
            ship_id: ship.id,
            ship_state: &ship.state,
            ship_position: Vec2::new(0.0, 0.0),
            ship_next_waypoint: None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Serialization

pub type ModuleInnerBox = Box<IModule+'static>;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModuleClass {
    ProjectileWeapon,
    Shield,
    Engine,
    Solar,
    Command,
    Cabin,
    BeamWeapon,
    Repair,
    VolleyMissile,
}

impl<'de> Deserialize<'de> for ModuleInnerBox {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use self::ModuleClass::*;
        use serde::de::{self, SeqAccess, Visitor};
        use std::fmt;

        struct ModuleInnerBoxVisitor;
        impl<'de> Visitor<'de> for ModuleInnerBoxVisitor {
            type Value = ModuleInnerBox;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "Expected a seq containing exactly 2 elements")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let module_class: ModuleClass = seq.next_element()?.unwrap();
                
                match module_class {
                    ProjectileWeapon => {
                        seq.next_element::<ProjectileWeaponModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            })
                    },
                    Shield => {
                        seq.next_element::<ShieldModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            })
                    },
                    Engine => {
                        let result = seq.next_element::<EngineModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            });
                        result
                    },
                    Solar => {
                        seq.next_element::<SolarModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            })
                    },
                    Command => {
                        seq.next_element::<CommandModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            })
                    },
                    Cabin => {
                        seq.next_element::<CabinModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            })
                    },
                    BeamWeapon => {
                        seq.next_element::<BeamWeaponModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            })
                    },
                    Repair => {
                        seq.next_element::<RepairModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            })
                    },
                    VolleyMissile => {
                        seq.next_element::<VolleyMissileModule>()?
                            .map(|x| Box::new(x) as Box<dyn IModule>)
                            .ok_or_else(|| {
                                de::Error::custom("Failed to deserialize EngineModule")
                            })
                    },
                }
            }
        }

        d.deserialize_tuple(2, ModuleInnerBoxVisitor)
    }
}

impl Serialize for ModuleInnerBox {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use std::mem;
        use std::raw;
        use serde::ser::SerializeTuple;
        
        use self::ModuleClass::*;

        let mut tup = s.serialize_tuple(2)?;
        
        let module_class = self.get_class();
        tup.serialize_element(&module_class)?;
        
        match module_class {
            ProjectileWeapon => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                tup.serialize_element(mem::transmute::<_, &ProjectileWeaponModule>(to.data))?;
            },
            Shield => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                tup.serialize_element(mem::transmute::<_, &ShieldModule>(to.data))?;
            },
            Engine => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                let module: &EngineModule = mem::transmute(to.data);
                tup.serialize_element(module)?;
            },
            Solar => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                tup.serialize_element(mem::transmute::<_, &SolarModule>(to.data))?;
            },
            Command => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                tup.serialize_element(mem::transmute::<_, &CommandModule>(to.data))?;
            },
            Cabin => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                tup.serialize_element(mem::transmute::<_, &CabinModule>(to.data))?;
            },
            BeamWeapon => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                tup.serialize_element(mem::transmute::<_, &BeamWeaponModule>(to.data))?;
            },
            Repair => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                tup.serialize_element(mem::transmute::<_, &RepairModule>(to.data))?;
            },
            VolleyMissile => unsafe {
                let to: raw::TraitObject = mem::transmute(self.deref());
                tup.serialize_element(mem::transmute::<_, &VolleyMissileModule>(to.data))?;
            },
        }

        tup.end()
    }
}
