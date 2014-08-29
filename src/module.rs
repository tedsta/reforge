use std::io::IoResult;

use net::{InPacket, OutPacket};
use sim_element::SimElement;

pub trait Module {
    fn create_sim_elements(&self) -> Vec<Box<SimElement>>;
}

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
    
    pub fn new_from_packet(packet: &mut InPacket) -> IoResult<ModuleBase> {
        Ok(ModuleBase {
            power: try!(packet.read_u32()),
            max_power: try!(packet.read_u32()),
            damage: try!(packet.read_u32()),
            hull: try!(packet.read_u32())
        })
    }
    
    pub fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write_u32(self.power));
        try!(packet.write_u32(self.max_power));
        try!(packet.write_u32(self.damage));
        try!(packet.write_u32(self.hull));
        Ok(())
    }
}