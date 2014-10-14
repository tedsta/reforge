use rsfml::window::{keyboard, mouse, event};
use rsfml::graphics::RenderWindow;

use render::Renderer;

pub struct SpaceGui;

impl SpaceGui {
    pub fn new() -> SpaceGui {
        SpaceGui
    }
    
    pub fn update(&mut self, window: &mut RenderWindow) {
        loop {
            match window.poll_event() {
                event::Closed => window.close(),
                event::KeyPressed{code, ..} => match code {
                    keyboard::Escape => {},
                    _ => {},
                },
                event::KeyReleased{..} => {},
                event::MouseButtonPressed{button, x, y} => {
                }
                event::MouseButtonReleased{button, x, y} => {
                }
                event::NoEvent => break,
                _ => {}
            };
        }
    }
    
    pub fn draw(&self, renderer: &mut Renderer) {
    }
}