use std::io::{IoResult, IoError, InvalidInput};

use net::{InPacket, OutPacket, Packable};
use render::Renderer;
use sim_element::SimElement;

// Use+reexport all of the modules
pub use self::engine::EngineModule;

pub mod engine;

///////////////////////////////////////////////////////////////////////////////////////////////////

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

impl SimElement for Module {
    fn on_simulation_begin(&mut self) {
        match *self {
            Engine(mut m) => m.on_simulation_begin(),
        }
    }
    
    fn on_simulation_time(&mut self, time: f32) {
         match *self {
            Engine(mut m) => m.on_simulation_time(time),
        }
    }
    
    fn on_simulation_end(&mut self) {
         match *self {
            Engine(mut m) => m.on_simulation_end(),
        }
    }
    
    fn draw(&self, renderer: &mut Renderer, simulating: bool) {
        match *self {
            Engine(m) => m.draw(renderer, simulating),
        }
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
        match *self {
            Engine(m) => m.write_plans(packet),
        }
    }
    
    fn read_plans(&self, packet: &mut InPacket) {
        match *self {
            Engine(m) => m.read_plans(packet),
        }
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
        match *self {
            Engine(m) => m.write_results(packet),
        }
    }
    
    fn read_results(&self, packet: &mut InPacket) {
        match *self {
            Engine(m) => m.read_results(packet),
        }
    }
}

pub fn read_module_from_packet(packet: &mut InPacket) -> IoResult<Module> {
    let module_type = try!(packet.read_u16());
    match module_type {
        0 => {
            Ok(Engine(try!(packet.read::<EngineModule>())))
        },
        _ => {Err(IoError{kind: InvalidInput, desc: "Failed to read module with invalid module type ID", detail: None})}
    }
}

pub fn write_module_to_packet(module: &Module, packet: &mut OutPacket) -> IoResult<()> {
    packet.write_u16(module.type_id()).unwrap();
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