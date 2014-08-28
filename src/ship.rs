use module::Module;
use net::{InPacket, OutPacket};

pub struct Ship {
    modules: Vec<Box<Module>>,
}

impl Ship {
    pub fn new() -> Ship {
        Ship{modules: vec!()}
    }
    
    pub fn new_from_packet(packet: &mut InPacket) -> Ship {
        let ship = Ship::new();
        ship
    }
    
    pub fn to_packet(&self) -> OutPacket {
        let packet = OutPacket::new();
        packet
    }
}