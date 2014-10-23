use assets::ENGINE_TEXTURE;
use module::{IModule, Module, ModuleBase, Propulsion, Engine};
use net::{InPacket, OutPacket};
use ship::ShipState;
use sim::SimEventAdder;

#[deriving(Encodable, Decodable)]
pub struct EngineModule {
    pub base: ModuleBase,
}

impl EngineModule {
    pub fn new() -> Module {
        Engine(EngineModule {
            base: ModuleBase::new(Propulsion, ENGINE_TEXTURE),
        })
    }
}

impl IModule for EngineModule {
    fn server_preprocess(&mut self, ship_state: &mut ShipState) {
    }
    
    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder) {
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
    }
    
    fn read_plans(&mut self, packet: &mut InPacket) {
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
    }
}