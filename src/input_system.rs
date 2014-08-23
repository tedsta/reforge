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

pub struct InputSystem<'r> {
    key_handlers: Vec<&'r mut KeyHandler>,
    mouse_handlers: Vec<&'r mut MouseHandler>,
}

impl<'r> InputSystem<'r> {
    pub fn new() -> InputSystem<'r> {
        InputSystem{key_handlers: vec!(), mouse_handlers: vec!()}
    }
    
    pub fn add_key_handler<T: KeyHandler+'static>(&mut self, handler: &'r mut T) {
        self.key_handlers.push(handler);
    }
    
    pub fn add_mouse_handler<T: MouseHandler+'static>(&mut self, handler: &'r mut T) {
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