use event::GenericEvent;
use graphics::Context;
use input::{mouse, Button};
use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;

use module::{ModelIndex, ModuleIndex};

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum ShipEditAction {
    Place(ModelIndex),
    Remove(ModuleIndex),
}

pub struct ShipEditGui {
    action: Option<ShipEditAction>,
}

impl ShipEditGui {
    pub fn new() -> ShipEditGui {
        ShipEditGui {
            action: None,
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2]) -> Option<ShipEditAction> {
        use event::*;
        
        e.press(|button| {
            match button {
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => { self.on_mouse_left_pressed(mouse_pos, button); },
                        mouse::MouseButton::Right => { },
                        _ => {},
                    }
                },
                _ => {},
            }
        });
        
        self.action.take()
    }

    fn on_mouse_left_pressed(&mut self, mouse_pos: [f64; 2], button: mouse::MouseButton) {
    }

    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use quack::Set;
        use graphics::*;
        use graphics::text::Text;
        
        // Render background window
        Rectangle::new([0.2, 0.05, 0.3, 0.8])
            .draw([0.0, 0.0, 400.0, 450.0], &context.draw_state, context.transform, gl);
        
        // Label text
        {
            let context = context.trans(5.0, 20.0);
            Text::colored([1.0; 4], 15).draw(
                "module inventory",
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
        }
    }
}