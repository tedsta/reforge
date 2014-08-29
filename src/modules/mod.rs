use std::io::{IoResult, IoError, InvalidInput};

use net::{InPacket};
use module::Module;

// Use+reexport all of the modules
pub use self::engine::EngineModule;

pub mod engine;

#[deriving(FromPrimitive)]
pub enum ModuleType {
    Engine,
}

fn module_from_packet(packet: &mut InPacket) -> IoResult<Box<Module>> {
    let module_type: ModuleType = match FromPrimitive::from_u16(try!(packet.read_u16())) {
        Some(module_type) => module_type,
        None => return Err(IoError{kind: InvalidInput, desc: "Unknown module type", detail: None})
    };
    match module_type {
        Engine => Ok(box try!(EngineModule::new_from_packet(packet)) as Box<Module>),
    }
}