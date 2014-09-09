use std::io::IoResult;

use module::{Module, ModuleBase, Engine};
use net::{InPacket, OutPacket, Packable};

pub struct EngineModule {
    base: ModuleBase,
}

impl EngineModule {
    pub fn new() -> Module {
        Engine(EngineModule{base: ModuleBase::new()})
    }
}

impl Packable for EngineModule {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<EngineModule> {
        let base = try!(packet.read());

        Ok(EngineModule {
            base: base,
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write(&self.base));
        Ok(())
    }
}