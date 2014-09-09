use std::rc::Rc;
use std::cell::RefCell;

use std::io::{IoResult, IoError, InvalidInput};

use net::{InPacket, OutPacket, Packable};
use sim_element::SimElement;

// Use+reexport all of the modules
pub use self::engine::EngineModule;

pub mod engine;

///////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleRef = Rc<RefCell<Module>>;

pub enum Module {
    Engine(EngineModule),
}

impl Module {
    fn type_id(&self) -> u16 {
        match *self {
            Engine(_) => 0,
        }
    }
}

pub fn read_module_from_packet(packet: &mut InPacket) -> IoResult<Module> {
    let module_type = try!(packet.read_u16());
    match module_type {
        0 => {
            Ok(Engine(try!(packet.read::<EngineModule>())))
        },
        _ => {fail!("Failed to read module with invalid module type id {}", module_type)}
    }
}

pub fn write_module_to_packet(module: &Module, packet: &mut OutPacket) -> IoResult<()> {
    packet.write_u16(module.type_id());
    match *module {
        Engine(module) => try!(module.write_to_packet(packet)),
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////////////////////////

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