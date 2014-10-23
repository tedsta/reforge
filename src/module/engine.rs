use assets::ENGINE_TEXTURE;
use module::{IModule, Module, ModuleBase, Propulsion, Engine};
use net::{InPacket, OutPacket};
use ship::{ShipId, ShipState};
use sim::SimEventAdder;

#[cfg(client)]
use sim::SimVisuals;

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
    
    #[cfg(client)]
    fn add_sim_visuals(&self, _: ShipId, _: &mut SimVisuals) {
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