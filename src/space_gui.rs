use rsfml::window::{keyboard, mouse, event};
use rsfml::graphics::{RenderTarget, RenderWindow};

use assets::LASER_TEXTURE;
use module::{MODULE_CATEGORIES, ModuleCategory};
use sfml_renderer::SfmlRenderer;
use ship::Ship;

pub struct SpaceGui {
    module_category: Option<ModuleCategory>, // Selected module category
}

impl SpaceGui {
    pub fn new() -> SpaceGui {
        SpaceGui {
            module_category: None,
        }
    }
    
    pub fn update(&mut self, window: &mut RenderWindow) {
        loop {
            match window.poll_event() {
                event::Closed => window.close(),
                event::KeyPressed{code, ..} => match code {
                    keyboard::Escape => { window.close(); },
                    code => { self.on_key_pressed(code); },
                },
                event::KeyReleased{..} => {},
                event::MouseButtonPressed{button, x, y} => {
                    match button {
                        mouse::MouseLeft => self.on_mouse_left_pressed(x, y),
                        _ => {},
                    }
                }
                /*event::MouseButtonReleased{button, x, y} => {
                }*/
                event::NoEvent => break,
                _ => {}
            };
        }
    }
    
    pub fn draw<T: RenderTarget>(&self, renderer: &SfmlRenderer<T>, client_ship: &Ship) {
        for category in MODULE_CATEGORIES.iter() {
            let icon_y: f32 =
                match self.module_category {
                    Some(c) if c == category.id => 584.0,
                    _ => { 600.0 },
                };
            
            renderer.draw_texture(LASER_TEXTURE, 10.0 + (64.0*(category.id as u8 as f32)), icon_y);
        }
        
        match self.module_category {
            Some(category) => {
                let mut i = 0u8;
                for module in client_ship.modules.iter() {
                    if module.borrow().get_base().category == category {                    
                        renderer.draw_texture(LASER_TEXTURE, 10.0 + (64.0*(i as f32)), 500.0);
                        i += 1;
                    }
                }
            },
            None => {},
        }
    }
    
    pub fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    pub fn on_mouse_left_pressed(&mut self, x: i32, y: i32) {
        for category in MODULE_CATEGORIES.iter() {
            let icon_x = 10 + (64*(category.id as i32));
            let icon_y: i32 =
                match self.module_category {
                    Some(c) if c == category.id => 584,
                    _ => { 600 },
                };
            let icon_w = 48;
            let icon_h = 48;
            
            if x >= icon_x && x <= icon_x+icon_w && y >= icon_y && y <= icon_y+icon_h {
                match self.module_category {
                    // Reclicked selected module category: deselect it
                    Some(c) if c == category.id => self.module_category = None,
                    // Selected a new module category
                    _ => self.module_category = Some(category.id),
                }
                break;
            }
        }
    }
}