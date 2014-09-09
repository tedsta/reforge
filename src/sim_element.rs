use net::{InPacket, OutPacket};

pub trait SimElement {
    fn on_simulation_begin(&mut self) {}
    fn on_simulation_time(&mut self, f32) {}
    fn on_simulation_end(&mut self) {}
    
    fn write_plans(&self, &mut OutPacket) {}
    fn read_plans(&self, &mut InPacket) {}
}