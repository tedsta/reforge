use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::cell::RefCell;

use battle_context::BattleContext;
use ship::{ShipId, ShipIndex, ShipState};

// SimVisual imports
#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::GlGraphics;
#[cfg(feature = "client")]
use sdl2_mixer;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait SimEvent {
    fn apply(&mut self, ship_state: &mut ShipState);
}

pub struct SimEvents<'a> {
    events: Vec<Vec<(ShipIndex, Box<SimEvent+'a>)>>, // events[tick][event]
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
    
    pub fn apply_tick(&mut self, bc: &mut BattleContext, tick: u32) {
        let tick = tick as usize;
        for (ship, mut event) in self.events[tick].drain(..) {
            event.apply(&mut ship.get_mut(bc).state);
        }
    }
    
    pub fn add(&mut self, tick: u32, ship: ShipIndex, event: Box<SimEvent+'a>) {
        self.events[tick as usize].push((ship, event));
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "client")]
static NUM_LAYERS: u8 = 4;

#[cfg(feature = "client")]
pub trait SimVisual {
    fn draw(&mut self, context: &Context, gl: &mut GlGraphics, time: f64);
}

#[cfg(feature = "client")]
pub struct SimEffects<'a> {
    effects: [Vec<(ShipId, Box<SimVisual+'a>)>; 4],
    
    // Audio stuff
    sounds: Vec<(f64, isize, Rc<RefCell<sdl2_mixer::Chunk>>)>,
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
    
    pub fn add_visual<V: SimVisual+'a>(&mut self, ship: ShipId, layer: u8, visual: V) {
        if layer >= NUM_LAYERS {
            panic!("Tried to add visual to layer {} when only {} layers exist", layer, NUM_LAYERS);
        }
        self.effects[layer as usize].push((ship, Box::new(visual)));
    }
    
    pub fn add_sound(&mut self, time: f64, loops: isize, sound: Rc<RefCell<sdl2_mixer::Chunk>>) {
        let mut index = 0;
        for &(sound_time, _, _) in self.sounds.iter() {
            if sound_time > time {
                break;
            }
            index += 1;
        }
        
        self.sounds.insert(index, (time, loops, sound));
    }
    
    pub fn update(&mut self, context: &Context, gl: &mut GlGraphics, ship: ShipId, time: f64) {
        use std::default::Default;
    
        while self.next_sound < self.sounds.len() {
            let (sound_time, loops, ref sound) = self.sounds[self.next_sound];
            if sound_time > time {
                break;
            }
            let sound_group: sdl2_mixer::Group = Default::default();
            if let Some(channel) = sound_group.find_available() {
                
                channel.play(sound.borrow().deref(), loops);
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
