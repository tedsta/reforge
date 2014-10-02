use std::collections::HashMap;
use std::io::IoResult;

use module::{Module, ModuleBase, Engine};
use net::{ClientId, InPacket, OutPacket, Packable};
use render;
use render::{Renderer, TextureId};
use ship::Ship;
use sim_element::SimElement;

pub struct EngineModule {
    pub base: ModuleBase,
}

impl EngineModule {
    pub fn new() -> Module {
        Engine(EngineModule {
            base: ModuleBase::new(),
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
    fn server_preprocess(&mut self, ships: &HashMap<ClientId, Ship>) {
    }
    
    fn before_simulation(&mut self, ships: &HashMap<ClientId, Ship>) {
    }
    
    fn on_simulation_time(&mut self, ships: &HashMap<ClientId, Ship>, tick: u32) {
        println!("Simulating module at {}", tick);
    }
    
    fn after_simulation(&mut self, ships: &HashMap<ClientId, Ship>) {
    }
    
    fn draw(&mut self, renderer: &mut Renderer, simulating: bool, time: f32) {
        renderer.draw_texture(render::Engine, (self.base.x as f32)*(48f32) + (time*100f32), (self.base.y as f32)*(48f32));
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