use std::rc::Rc;
use std::cell::RefCell;
use std::io::IoResult;

use module::{ModuleRef, Module, read_module_from_packet, write_module_to_packet};
use net::{InPacket, OutPacket, Packable};
use self::ship_gen::generate_ship;

// Use the ship_gen module privately here
mod ship_gen;

// Holds everything about the ship's damage, capabilities, etc.
pub struct ShipState {
    pub engines: uint,
    pub shields: uint,
    pub max_shields: uint,
}

impl ShipState {
    pub fn new() -> ShipState {
        ShipState{engines: 0, shields: 0, max_shields: 0}
    }
}

impl Packable for ShipState {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<ShipState> {
        Ok(ShipState {
            engines: try!(packet.read_u32()) as uint,
            shields: try!(packet.read_u32()) as uint,
            max_shields: try!(packet.read_u32()) as uint,
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write_u32(self.engines as u32));
        try!(packet.write_u32(self.shields as u32));
        try!(packet.write_u32(self.max_shields as u32));
        Ok(())
    }
}

pub struct Ship {
    pub state: ShipState,
    pub modules: Vec<ModuleRef>,
}

impl Ship {
    pub fn new() -> Ship {
        Ship{state: ShipState::new(), modules: vec!()}
    }
    
    pub fn generate() -> Ship {
        generate_ship()
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module(&mut self, module: Module) -> bool {
        self.modules.push(Rc::new(RefCell::new(module)));
        true
    }
}

impl Packable for Ship {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<Ship> {
        let state: ShipState = try!(packet.read());
        
        // Deserialize modules
        let num_modules = try!(packet.read_u8());
        let mut modules = vec!();
        
        while modules.len() < num_modules as uint {
            modules.push(Rc::new(RefCell::new(try!(read_module_from_packet(packet)))));
        }
        
        Ok(Ship {
            state: state,
            modules: modules,
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(self.state.write_to_packet(packet));
        try!(packet.write_u8(self.modules.len() as u8));
        for module in self.modules.iter() {
            try!(write_module_to_packet(module.borrow().deref(), packet));
        }
        Ok(())
    }
}