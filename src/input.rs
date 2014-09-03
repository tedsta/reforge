use rsfml::window::{keyboard, mouse, event};
use rsfml::graphics::RenderWindow;

pub trait KeyHandler {
    fn on_key_pressed(&mut self, key: keyboard::Key);
    fn on_key_released(&mut self, key: keyboard::Key);
}

pub trait MouseHandler {
    fn on_mouse_button_pressed(&mut self, mouse::MouseButton);
    fn on_mouse_button_released(&mut self, mouse::MouseButton);
}

// Type aliases for boxed KeyHandler and MouseHandler
pub type KeyHandlerBox = Box<KeyHandler + 'static>;
pub type MouseHandlerBox = Box<MouseHandler + 'static>;

pub struct InputSystem {
    key_handlers: Vec<KeyHandlerBox>,
    mouse_handlers: Vec<MouseHandlerBox>,
}

impl InputSystem {
    pub fn new() -> InputSystem {
        InputSystem{key_handlers: vec!(), mouse_handlers: vec!()}
    }
    
    pub fn add_key_handler(&mut self, handler: KeyHandlerBox) {
        self.key_handlers.push(handler);
    }
    
    pub fn add_mouse_handler(&mut self, handler: MouseHandlerBox) {
        self.mouse_handlers.push(handler);
    }
    
    pub fn update(&mut self, window: &mut RenderWindow) {
        loop {
            match window.poll_event() {
                event::Closed => window.close(),
                event::KeyPressed{code, ..} => match code {
                    keyboard::Escape => {window.close(); break},
                    _ => {
                        for handler in self.key_handlers.mut_iter() {
                            handler.on_key_pressed(code);
                        }
                    }
                },
                event::KeyReleased{code, ..} => {
                    for handler in self.key_handlers.mut_iter() {
                        handler.on_key_released(code);
                    }
                },
                event::NoEvent => break,
                _ => {}
            }
        }
    }
}