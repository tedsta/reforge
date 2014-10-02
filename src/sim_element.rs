use std::collections::HashMap;

use net::{ClientId, InPacket, OutPacket};
use render::Renderer;
use ship::Ship;

pub trait SimElement {
    fn server_preprocess(&mut self, ships: &HashMap<ClientId, Ship>);

    fn before_simulation(&mut self, ships: &HashMap<ClientId, Ship>);
    fn on_simulation_time(&mut self, ships: &HashMap<ClientId, Ship>, tick: u32);
    fn after_simulation(&mut self, ships: &HashMap<ClientId, Ship>);
    fn get_critical_times(&self) -> Vec<u32> {
        vec!()
    }
    
    fn draw(&mut self, renderer: &mut Renderer, simulating: bool, time: f32);
    
    fn write_plans(&self, packet: &mut OutPacket);
    fn read_plans(&self, packet: &mut InPacket);
    
    fn write_results(&self, packet: &mut OutPacket);
    fn read_results(&self, packet: &mut InPacket);
}