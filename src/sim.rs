use module::{Module, ModuleRef};

// SimVisual imports
#[cfg(client)]
use sfml_renderer::SfmlRenderer;
#[cfg(client)]
use ship::ShipId;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SimEvents<'a> {
    events: Vec<Vec<(ModuleRef, |&mut Module|: 'a)>>,
}

impl<'a> SimEvents<'a> {
    pub fn new() -> SimEvents<'a> {
        let mut events = Vec::with_capacity(100);
        while events.len() < 100 {
            events.push(vec!());
        }
        SimEvents {
            events: events,
        }
    }
    
    pub fn apply_tick(&mut self, tick: u32) {
        let tick = tick as uint;
        while self.events[tick].len() > 0 {
            let (module, event) = self.events.get_mut(tick).pop().unwrap();
            event(module.borrow_mut().deref_mut());
        }
    }
    
    pub fn create_adder<'b>(&'b mut self, module: ModuleRef) -> SimEventAdder<'a, 'b> {
        SimEventAdder {
            sim_events: self,
            module: module,
        }
    }
}

pub struct SimEventAdder<'a: 'b, 'b> {
    sim_events: &'b mut SimEvents<'a>,
    module: ModuleRef,
}

impl<'a, 'b> SimEventAdder<'a, 'b> {
    pub fn add(&mut self, tick: u32, event: |&mut Module|: 'a) {
        self.sim_events.events.get_mut(tick as uint).push((self.module.clone(), event));
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(client)]
pub struct SimVisuals<'a> {
    visuals: Vec<(ShipId, |&SfmlRenderer, f32|: 'a)>,
}

#[cfg(client)]
impl<'a> SimVisuals<'a> {
    pub fn new() -> SimVisuals<'a> {
        SimVisuals{visuals: vec!()}
    }
    
    pub fn add(&mut self, ship: ShipId, visual: |&SfmlRenderer, f32|: 'a) {
        self.visuals.push((ship, visual));
    }
    
    pub fn draw(&mut self, renderer: &SfmlRenderer, ship: ShipId, time: f32) {
        for &(v_ship, ref mut visual) in self.visuals.iter_mut() {
            if v_ship == ship { 
                (*visual)(renderer, time);
            }
        }
    }
}