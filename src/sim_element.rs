use battle_state::BattleContext;
use net::{InPacket, OutPacket};
use render::Renderer;

pub trait SimElement {
    fn server_preprocess(&mut self, context: &BattleContext);

    fn before_simulation(&mut self, context: &BattleContext);
    fn on_simulation_time(&mut self, context: &BattleContext, tick: u32);
    fn after_simulation(&mut self, context: &BattleContext);
    fn get_critical_times(&self) -> Vec<u32> {
        vec!()
    }
    
    fn draw(&mut self, renderer: &mut Renderer, context: &BattleContext, simulating: bool, time: f32);
    
    fn write_plans(&self, packet: &mut OutPacket);
    fn read_plans(&self, packet: &mut InPacket);
    
    fn write_results(&self, packet: &mut OutPacket);
    fn read_results(&self, packet: &mut InPacket);
}