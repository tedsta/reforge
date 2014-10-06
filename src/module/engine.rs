use std::collections::HashMap;
use std::io::IoResult;

use module::{Module, ModuleBase, Engine};
use net::{ClientId, InPacket, OutPacket, Packable};
use render::{Renderer, TextureId, ENGINE_TEXTURE};
use ship::ShipRef;
use sim_element::SimElement;

pub struct EngineModule {
    pub base: ModuleBase,
}

impl EngineModule {
    pub fn new() -> Module {
        Engine(EngineModule {
            base: ModuleBase::new(ENGINE_TEXTURE),
        })
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

impl SimElement for EngineModule {
    fn server_preprocess(&mut self, ships: &HashMap<ClientId, ShipRef>) {
    }
    
    fn before_simulation(&mut self, ships: &HashMap<ClientId, ShipRef>) {
    }
    
    fn on_simulation_time(&mut self, ships: &HashMap<ClientId, ShipRef>, tick: u32) {
    }
    
    fn after_simulation(&mut self, ships: &HashMap<ClientId, ShipRef>) {
    }
    
    fn draw(&mut self, renderer: &mut Renderer, simulating: bool, time: f32) {
        self.base.draw(renderer);
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
    }
    
    fn read_plans(&self, packet: &mut InPacket) {
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
    }
    
    fn read_results(&self, packet: &mut InPacket) {
    }
}