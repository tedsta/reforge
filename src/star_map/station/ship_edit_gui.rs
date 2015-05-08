use event::GenericEvent;
use graphics::Context;
use input::{mouse, Button};
use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;

use module::{ModelIndex, ModelStore};
use super::ShipEditAction;

pub type ModuleInventory = Vec<(String, Vec<(ModelIndex, u16)>)>;

pub struct ShipEditGui<'a> {
    model_store: &'a ModelStore,
    
    inventory: ModuleInventory,

    action: Option<ShipEditAction>,
}

impl<'a> ShipEditGui<'a> {
    pub fn new(model_store: &'a ModelStore, inventory: ModuleInventory) -> ShipEditGui<'a> {
        ShipEditGui {
            model_store: model_store,
            
            inventory: inventory,
        
            action: None,
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2]) -> Option<ShipEditAction> {
        use event::*;
        
        e.press(|button| {
            match button {
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => {
                            self.on_mouse_left_pressed(mouse_pos, button);
                        },
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
        let context = context.trans(5.0, 30.0);
        Text::colored([1.0; 4], 25).draw(
            "module inventory",
            glyph_cache,
            &context.draw_state, context.transform,
            gl,
        );
        
        // Draw the inventory
        let mut context = context.trans(0.0, 5.0);
        for &(ref category, ref modules) in &self.inventory {
            Rectangle::new([0.0, 1.0, 0.0, 1.0])
                .draw([0.0, 0.0, 75.0, 19.0], &context.draw_state, context.transform, gl);
            
            // Category label
            {
                let context = context.trans(5.0, 17.0);
                Text::colored([1.0; 4], 15).draw(
                    category,
                    glyph_cache,
                    &context.draw_state, context.transform,
                    gl,
                );
            }
            
            // Draw module icons
            {
                let mut context = context.trans(5.0, 24.0);
                
                for &(model, count) in modules {
                    Rectangle::new([1.0, 0.0, 0.0, 1.0])
                        .draw([0.0, 0.0, 48.0, 48.0], &context.draw_state, context.transform, gl);
                    
                    context.trans(50.0, 0.0);
                }
            }
            
            context = context.trans(77.0, 0.0);
        }
    }
}