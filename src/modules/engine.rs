use std::io::IoResult;

use module::{Module, ModuleBase};
use net::{InPacket, OutPacket};
use sim_element::SimElement;

pub struct EngineModule {
    base: ModuleBase,
}

impl EngineModule {
    pub fn new() -> EngineModule {
        EngineModule{base: ModuleBase::new()}
    }
    
    pub fn new_from_packet(packet: &mut InPacket) -> IoResult<EngineModule> {
        let engine = EngineModule::new();
        Ok(engine)
    }
    
    pub fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        Ok(())
    }
}

impl Module for EngineModule {
    fn create_sim_elements(&self) -> Vec<Box<SimElement>> {
        vec!()
    }
}