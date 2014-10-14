use std::cell::RefCell;
use std::rc::Rc;

use rsfml::window::{keyboard, mouse, event};
use rsfml::graphics::RenderWindow;

pub enum KeyEvent {
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
}

pub enum MouseEvent {
    MouseButtonPressed(mouse::MouseButton),
    MouseButtonReleased(mouse::MouseButton),
    MouseMoved(i32, i32),
}

pub struct InputSystem {
    key_senders: Vec<Sender<KeyEvent>>,
    mouse_senders: Vec<Sender<MouseEvent>>,
}

impl InputSystem {
    pub fn new() -> InputSystem {
        InputSystem{key_senders: vec!(), mouse_senders: vec!()}
    }
    
    pub fn add_key_receiver(&mut self) -> Receiver<KeyEvent> {
        let (sender, receiver) = channel();
        self.key_senders.push(sender);
        receiver
    }
    
    pub fn add_mouse_receiver(&mut self) -> Receiver<MouseEvent> {
        let (sender, receiver) = channel();
        self.mouse_senders.push(sender);
        receiver
    }
    
    pub fn update(&mut self, window: &mut RenderWindow) {
        loop {
            match window.poll_event() {
                event::Closed => window.close(),
                event::KeyPressed{code, ..} => match code {
                    keyboard::Escape => {window.close(); break},
                    _ => {
                        for sender in self.key_senders.iter() {
                            sender.send(KeyPressed(code));
                        }
                    }
                },
                event::KeyReleased{code, ..} => {
                    for sender in self.key_senders.iter() {
                        sender.send(KeyReleased(code));
                    }
                },
                event::NoEvent => break,
                _ => {}
            }
        }
    }
}