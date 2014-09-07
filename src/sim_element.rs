pub trait SimElement {
    fn on_simulation_begin(&mut self) {}
    fn set_simulation_time(&mut self, time: f32) {}
    fn on_simulation_end(&mut self) {}
}