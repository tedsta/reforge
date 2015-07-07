use piston::event::GenericEvent;
use graphics::Context;
use piston::input::{mouse, Button};
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;

use gui::TextButton;
use ship::Ship;
use vec::{Vec2, Vec2f};

pub enum NavMapGuiAction {
    Close,
}

pub struct NavMapGui {
    action: Option<NavMapGuiAction>,
    
    // Buttons
    close_button: TextButton,
}

impl NavMapGui {
    pub fn new() -> NavMapGui {
        NavMapGui {
            action: None,
            
            close_button: TextButton::new("Close".to_string(), 20, [450.0, 400.0], [150.0, 40.0]),
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2], client_ship: &mut Ship) -> Option<NavMapGuiAction> {
        use piston::event::*;
        
        e.press(|button| {
            match button {
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => { self.on_mouse_left_pressed(mouse_pos, button, client_ship); },
                        mouse::MouseButton::Right => { },
                        _ => {},
                    }
                },
                _ => {},
            }
        });
        
        // Handle buttons
        self.close_button.event(e, mouse_pos);
        
        if self.close_button.get_clicked() {
            self.action = Some(NavMapGuiAction::Close);
        }
        
        self.action.take()
    }

    fn on_mouse_left_pressed(&mut self, mouse_pos: [f64; 2], button: mouse::MouseButton, client_ship: &mut Ship) {
        let mouse_pos = Vec2 { x: mouse_pos[0] - 5.0, y: mouse_pos[1] - 25.0 };
    
        // TODO: see if they clicked something
    }

    pub fn draw(&mut self, context: &Context, gl: &mut GlGraphics, glyph_cache: &mut GlyphCache, client_ship: &Ship) {
        use graphics::*;
        use graphics::text::Text;
        
        Rectangle::new([0.2, 0.05, 0.3, 0.8])
            .draw([0.0, 0.0, 800.0, 450.0], &context.draw_state, context.transform, gl);
        
        // Render all the stuff in the nav map
        {
            let ref context = context.trans(5.0, 25.0);
        
            Rectangle::new([0.0, 0.0, 0.0, 1.0])
                .draw([0.0, 0.0, 800.0 - 10.0, 400.0 - 30.0], &context.draw_state, context.transform, gl);
            
            // TODO: draw stuff
        }
        
        {
            let context = context.trans(5.0, 20.0);
            Text::colored([1.0; 4], 15).draw(
                "nav map",
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
        }
        
        // Draw the buttons
        self.close_button.draw(context, gl, glyph_cache);
    }
}