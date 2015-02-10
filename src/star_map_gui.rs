use std::cell::RefCell;

use sdl2_window::Sdl2Window;
use event::{Events, GenericEvent};
use graphics::{Context};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use gui::TextButton;

pub struct StarMapGui {
    done: bool,
    
    // Buttons
    close_button: TextButton,
    jump_button: TextButton,
}

impl StarMapGui {
    pub fn new() -> StarMapGui {
        StarMapGui {
            done: false,
            
            close_button: TextButton::new("Close".to_string(), 20, [450.0, 400.0], [150.0, 40.0]),
            jump_button: TextButton::new("Jump".to_string(), 20, [610.0, 400.0], [150.0, 40.0]),
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2]) {
        use event::*;
        
        e.press(|button| {
            match button {
                Button::Mouse(button) => {
                    self.on_mouse_pressed(button);
                },
                _ => {},
            }
        });
        
        // Handle buttons
        self.jump_button.event(e, mouse_pos);
        self.close_button.event(e, mouse_pos);
        
        if self.close_button.get_clicked() {
            self.done = true;
        }
        
        if self.jump_button.get_clicked() {
            self.done = true;
        }
    }

    fn on_mouse_pressed(&mut self, button: mouse::MouseButton) {
        match button {
            mouse::MouseButton::Left => {},
            mouse::MouseButton::Right => {},
            _ => {},
        }
    }

    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use quack::Set;
        use graphics::*;
        use graphics::text::Text;
        
        Rectangle::new([0.2, 0.05, 0.3, 0.8])
            .draw([0.0, 0.0, 800.0, 450.0], context, gl);
        
        Rectangle::new([0.0, 0.0, 0.0, 1.0])
            .draw([5.0, 25.0, 800.0 - 10.0, 400.0 - 30.0], context, gl);
        
        Text::colored([1.0; 4], 15).draw(
            "star map",
            glyph_cache,
            &context.trans(5.0, 20.0),
            gl,
        );
        
        // Draw the buttons
        self.close_button.draw(context, gl, glyph_cache);
        self.jump_button.draw(context, gl, glyph_cache);
    }
}