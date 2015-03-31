use std::marker::Reflect;
use std::ops::{Deref, DerefMut};
use std::any::TypeId;

use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use battle_state::BattleContext;
use module::{
    DamageVisual,
    IModule,
    Module,
    ModuleBase,
    ModuleBox,
    NetworkTarget,
    
    EngineModule,
    ProjectileWeaponModule,
    ShieldModule,
    SolarModule,
    CommandModule,
    BeamWeaponModule,
};

#[derive(RustcEncodable, RustcDecodable)]
pub struct ModuleBaseNetworked {
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
    
    pub target: Option<NetworkTarget>,
    pub plan_target: Option<NetworkTarget>,
    
    // Module damage visuals
    damage_visuals: Vec<DamageVisual>,
    
    pub index: u32, // Array index in ship. Used for referencing modules across network.
}

impl ModuleBaseNetworked {
    pub fn from_module_base(module_base: &ModuleBase) -> ModuleBaseNetworked {
        ModuleBaseNetworked {
            x: module_base.x,
            y: module_base.y,
            width: module_base.width,
            height: module_base.height,
            
            power: module_base.power,
            hp: module_base.hp,
            min_hp: module_base.min_hp,
            max_hp: module_base.max_hp,
            
            powered: module_base.powered,
            plan_powered: module_base.plan_powered,
            
            target: module_base.target.as_ref().map(|t| NetworkTarget::from_target(t)),
            plan_target: module_base.plan_target.as_ref().map(|t| NetworkTarget::from_target(t)),
            
            damage_visuals: module_base.damage_visuals.clone(),
            
            index: module_base.index,
        }
    }
    
    pub fn to_module_base(&self) -> (ModuleBase, Option<NetworkTarget>, Option<NetworkTarget>) {
        (ModuleBase {
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
            
            target: None,
            plan_target: None,
            
            damage_visuals: self.damage_visuals.clone(),
            
            index: self.index,
        }, self.target, self.plan_target)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleNetworkedBox(Box<IModuleNetworked + 'static>);

pub struct ModuleNetworked<M: IModule> {
    pub base: ModuleBaseNetworked,
    pub module: M,
}

pub trait IModuleNetworked : Send {
    fn get_type_id(&self) -> TypeId;
    fn get_base(&self) -> &ModuleBaseNetworked;
    fn get_module(&self) -> &IModule;

    fn to_module(&self) -> (ModuleBox, Option<NetworkTarget>, Option<NetworkTarget>);
}

impl ModuleNetworkedBox {
    pub fn new<M>(module: M) -> ModuleNetworkedBox
        where M: IModuleNetworked + 'static
    {
        ModuleNetworkedBox(Box::new(module))
    }
}

impl<M> IModuleNetworked for ModuleNetworked<M>
    where M: IModule+Reflect+Clone + 'static
{
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<M>()
    }
    
    fn get_base(&self) -> &ModuleBaseNetworked {
        &self.base
    }
    
    fn get_module(&self) -> &IModule {
        &self.module
    }
    
    fn to_module(&self) -> (ModuleBox, Option<NetworkTarget>, Option<NetworkTarget>) {
        let (base, target, plan_target) = self.base.to_module_base();
    
        (ModuleBox::new(Module{base: base, module: self.module.clone()}), target, plan_target)
    }
}

impl Deref for ModuleNetworkedBox {
    type Target = IModuleNetworked+'static;

    fn deref<'a>(&'a self) -> &'a (IModuleNetworked+'static) {
        let &ModuleNetworkedBox(ref module) = self;
        module.deref()
    }
}

impl DerefMut for ModuleNetworkedBox {
    fn deref_mut<'a>(&'a mut self) -> &'a mut (IModuleNetworked+'static) {
        let &mut ModuleNetworkedBox(ref mut module) = self;
        module.deref_mut()
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

impl Decodable for ModuleNetworkedBox {
    fn decode<D: Decoder>(d: &mut D) -> Result<ModuleNetworkedBox, D::Error> {
        use self::ModuleClass::*;
        
        let module_class: ModuleClass = try!(Decodable::decode(d));
        let base: ModuleBaseNetworked = try!(Decodable::decode(d));
        
        match module_class {
            ProjectileWeapon => Ok(ModuleNetworkedBox::new(ModuleNetworked {
                base: base,
                module: try!(<ProjectileWeaponModule as Decodable>::decode(d)),
            })),
            Shield => Ok(ModuleNetworkedBox::new(ModuleNetworked {
                base: base,
                module: try!(<ShieldModule as Decodable>::decode(d)),
            })),
            Engine => Ok(ModuleNetworkedBox::new(ModuleNetworked {
                base: base,
                module: try!(<EngineModule as Decodable>::decode(d)),
            })),
            Solar => Ok(ModuleNetworkedBox::new(ModuleNetworked {
                base: base,
                module: try!(<SolarModule as Decodable>::decode(d)),
            })),
            Command => Ok(ModuleNetworkedBox::new(ModuleNetworked {
                base: base,
                module: try!(<CommandModule as Decodable>::decode(d)),
            })),
            BeamWeapon => Ok(ModuleNetworkedBox::new(ModuleNetworked {
                base: base,
                module: try!(<BeamWeaponModule as Decodable>::decode(d)),
            })),
        }
    }
}

impl Encodable for ModuleNetworkedBox {
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