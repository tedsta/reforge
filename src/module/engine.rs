use std::collections::HashMap;
use std::io::IoResult;

use battle_state::BattleContext;
use module::{Module, ModuleBase, Engine};
use net::{ClientId, InPacket, OutPacket};
use render::{Renderer, TextureId, ENGINE_TEXTURE};
use sim_element::SimElement;

#[deriving(Encodable, Decodable)]
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

impl SimElement for EngineModule {
    fn server_preprocess(&mut self, context: &BattleContext) {
    }
    
    fn before_simulation(&mut self, context: &BattleContext) {
    }
    
    fn on_simulation_time(&mut self, context: &BattleContext, tick: u32) {
    }
    
    fn after_simulation(&mut self, context: &BattleContext) {
    }
    
    fn draw(&mut self, renderer: &mut Renderer, context: &BattleContext, simulating: bool, time: f32) {
        let ship_index = self.base.index.as_ref().expect("Module not attached to ship").ship;
        let ship = context.get_ship(&ship_index).expect(format!("Failed to get ship {}", ship_index).as_slice());
        self.base.draw(renderer, ship);
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