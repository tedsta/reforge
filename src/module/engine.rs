use std::io::IoResult;

use module::{Module, ModuleBase, Engine};
use net::{InPacket, OutPacket, Packable};
use render::Renderer;
use sim_element::SimElement;

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

impl SimElement for EngineModule {
    fn on_simulation_begin(&mut self) {
    }
    
    fn on_simulation_time(&mut self, time: f32) {
        println!("Simulating module at {}", time);
    }
    
    fn on_simulation_end(&mut self) {
    }
    
    fn draw_planning(&self, renderer: &mut Renderer) {
    }
    
    fn draw_simulating(&self, renderer: &mut Renderer) {
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