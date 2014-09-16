use net::{InPacket, OutPacket};
use render::Renderer;

pub trait SimElement {
    fn on_simulation_begin(&mut self);
    fn on_simulation_time(&mut self, time: f32);
    fn on_simulation_end(&mut self);
    
    fn draw(&self, renderer: &mut Renderer, simulating: bool);
    
    fn write_plans(&self, packet: &mut OutPacket);
    fn read_plans(&self, packet: &mut InPacket);
    
    fn write_results(&self, packet: &mut OutPacket);
    fn read_results(&self, packet: &mut InPacket);
}