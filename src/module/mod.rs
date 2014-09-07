use std::intrinsics::TypeId;

use std::io::{IoResult, IoError, InvalidInput};

use net::{InPacket, OutPacket, Packable};
use sim_element::SimElement;

// Use+reexport all of the modules
pub use self::engine::EngineModule;

pub mod engine;

///////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Module : ModuleTypeId + SimElement + Packable {
    fn create_sim_elements(&self) -> Vec<Box<SimElement>>;
}

// Type alias for boxed Module... because with the 'static it's too tedious
pub type ModuleBox = Box<Module + 'static>;

pub struct ModuleBase {
    power: u32,
    max_power: u32,
    damage: u32,
    hull: u32,
}

impl ModuleBase {
    pub fn new() -> ModuleBase {
        ModuleBase{power: 0, max_power: 1, damage: 0, hull: 0}
    }
}

impl Packable for ModuleBase {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<ModuleBase> {
        Ok(ModuleBase {
            power: try!(packet.read_u32()),
            max_power: try!(packet.read_u32()),
            damage: try!(packet.read_u32()),
            hull: try!(packet.read_u32())
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write_u32(self.power));
        try!(packet.write_u32(self.max_power));
        try!(packet.write_u32(self.damage));
        try!(packet.write_u32(self.hull));
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(FromPrimitive)]
pub enum ModuleType {
    Engine,
}

pub fn read_module_from_packet(packet: &mut InPacket) -> IoResult<ModuleBox> {
    let module_type: ModuleType = match FromPrimitive::from_u16(try!(packet.read_u16())) {
        Some(module_type) => module_type,
        None => return Err(IoError{kind: InvalidInput, desc: "Unknown module type", detail: None})
    };
    match module_type {
        Engine => {
            let module: Box<EngineModule> = box try!(packet.read());
            Ok(module as ModuleBox)
        },
    }
}

pub fn write_module_to_packet(module: &ModuleBox, packet: &mut OutPacket) -> IoResult<()> {
    if module.get_type_id() == TypeId::of::<EngineModule>() {
        try!(packet.write_u16(Engine as u16));
    }
    try!(module.write_to_packet(packet));
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////////////////////////

trait ModuleTypeId {
    /// Get the `TypeId` of `self`
    fn get_type_id(&self) -> TypeId;
}

impl<T: 'static> ModuleTypeId for T {
    fn get_type_id(&self) -> TypeId { TypeId::of::<T>() }
}