use rsfml::window::keyboard;

use input::{InputSystem, KeyEvent, MouseEvent};
use render::Renderer;

pub struct SpaceGui {
    key_receiver: Receiver<KeyEvent>,
    mouse_receiver: Receiver<MouseEvent>,
}

impl SpaceGui {
    pub fn new(input: &mut InputSystem) -> SpaceGui {
        SpaceGui {
            key_receiver: input.add_key_receiver(),
            mouse_receiver: input.add_mouse_receiver(),
        }
    }
    
    pub fn update(&mut self) {
    }
    
    pub fn draw(&self, renderer: &mut Renderer) {
    }
}