use std::ops::{DerefMut};
use std::rc::Rc;

use module::{ModuleRef, ModuleBox};

// SimVisual imports
#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;
#[cfg(feature = "client")]
use sdl2_mixer;
#[cfg(feature = "client")]
use ship::ShipId;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait SimEvent {
    fn apply(&mut self, &mut ModuleBox);
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
        let tick = tick as usize;
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
        self.sim_events.events[tick as usize].push((self.module.clone(), event));
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: Replace the SimVisual struct impl trait model with unboxed closures once they are stable

#[cfg(feature = "client")]
static NUM_LAYERS: u8 = 4;

#[cfg(feature = "client")]
pub trait SimVisual {
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64);
}

#[cfg(feature = "client")]
pub struct SimEffects<'a> {
    effects: [Vec<(ShipId, Box<SimVisual+'a>)>; 4],
    
    // Audio stuff
    sounds: Vec<(f64, isize, Rc<sdl2_mixer::Chunk>)>,
    next_sound: usize,
}

#[cfg(feature = "client")]
impl<'a> SimEffects<'a> {
    pub fn new() -> SimEffects<'a> {
        use std::default::Default;
    
        SimEffects {
            effects: [vec!(), vec!(), vec!(), vec!()],
            
            sounds: vec!(),
            next_sound: 0,
        }
    }
    
    pub fn add_visual(&mut self, ship: ShipId, layer: u8, visual: Box<SimVisual+'a>) {
        if layer >= NUM_LAYERS {
            panic!("Tried to add visual to layer {} when only {} layers exist", layer, NUM_LAYERS);
        }
        self.effects[layer as usize].push((ship, visual));
    }
    
    pub fn add_sound(&mut self, time: f64, loops: isize, sound: Rc<sdl2_mixer::Chunk>) {
        let mut index = 0;
        for &(sound_time, _, _) in self.sounds.iter() {
            if sound_time > time {
                break;
            }
            index += 1;
        }
        
        self.sounds.insert(index, (time, loops, sound));
    }
    
    pub fn update(&mut self, context: &Context, gl: &mut Gl, ship: ShipId, time: f64) {
        use std::default::Default;
    
        while self.next_sound < self.sounds.len() {
            let (sound_time, loops, ref sound) = self.sounds[self.next_sound];
            if sound_time > time {
                break;
            }
            let sound_group: sdl2_mixer::Group = Default::default();
            if let Some(channel) = sound_group.find_available() {
                
                channel.play(sound, loops);
            } else {
                println!("Failed to play sound");
            }
            self.next_sound += 1;
        }
    
        for layer in self.effects.iter_mut() {
            for &mut (v_ship, ref mut visual) in layer.iter_mut() {
                if v_ship == ship {
                    visual.draw(context, gl, time);
                }
            }
        }
    }
    
    pub fn reset(&mut self) {
        self.sounds.clear();
        self.next_sound = 0;
        for visual_vec in self.effects.iter_mut() {
            visual_vec.clear();
        }
    }
}
