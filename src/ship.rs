use std::io::IoResult;

use module::Module;
use net::{InPacket, OutPacket};
use ship_gen::generate_ship;

// Use the ship_gen module privately here
mod ship_gen;

pub struct Ship {
    modules: Vec<Box<Module>>,
}

impl Ship {
    pub fn new() -> Ship {
        Ship{modules: vec!()}
    }
    
    pub fn new_from_packet(packet: &mut InPacket) -> IoResult<Ship> {
        let ship = Ship::new();
        Ok(ship)
    }
    
    pub fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        Ok(())
    }
    
    pub fn generate() -> Ship {
        generate_ship()
    }
}