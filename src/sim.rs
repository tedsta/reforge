use std::ops::{DerefMut};

use module::{Module, ModuleRef};

// SimVisual imports
#[cfg(client)]
use piston::graphics::Context;
#[cfg(client)]
use opengl_graphics::Gl;
#[cfg(client)]
use ship::ShipId;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait SimEvent {
    fn apply(&mut self, &mut Module);
}

pub struct SimEvents<'a> {
    events: Vec<Vec<(ModuleRef, Box<SimEvent+'a>)>>,
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
            let (module, mut event) = self.events[tick].pop().unwrap();
            event.apply(module.borrow_mut().deref_mut());
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
    pub fn add(&mut self, tick: u32, event: Box<SimEvent+'a>) {
        self.sim_events.events[tick as uint].push((self.module.clone(), event));
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: Replace the SimVisual struct impl trait model with unboxed closures once they are stable

#[cfg(client)]
static NUM_LAYERS: u8 = 2;

#[cfg(client)]
pub trait SimVisual {
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64);
}

#[cfg(client)]
pub struct SimVisuals<'a> {
    visuals: [Vec<(ShipId, Box<SimVisual+'a>)>; 2],
}

#[cfg(client)]
impl<'a> SimVisuals<'a> {
    pub fn new() -> SimVisuals<'a> {
        SimVisuals{visuals: [vec!(), vec!()]}
    }
    
    pub fn add(&mut self, ship: ShipId, layer: u8, visual: Box<SimVisual+'a>) {
        if layer >= NUM_LAYERS {
            panic!("Tried to add visual to layer {} when only {} layers exist", layer, NUM_LAYERS);
        }
        self.visuals[layer as uint].push((ship, visual));
    }
    
    pub fn draw(&mut self, context: &Context, gl: &mut Gl, ship: ShipId, time: f64) {
        for layer in self.visuals.iter_mut() {
            for &mut (v_ship, ref mut visual) in layer.iter_mut() {
                if v_ship == ship {
                    visual.draw(context, gl, time);
                }
            }
        }
    }
    
    pub fn clear(&mut self) {
        for visual_vec in self.visuals.iter_mut() {
            visual_vec.clear();
        }
    }
}
