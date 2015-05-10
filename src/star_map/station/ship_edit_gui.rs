use event::GenericEvent;
use graphics::Context;
use input::{mouse, Button};
use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;

use module::{ModelIndex, ModelStore};
use ship::ShipStored;
use vec::{Vec2, Vec2f};

use super::ShipEditAction;

pub type ModuleInventory = Vec<(String, Vec<(ModelIndex, u16)>)>;

pub struct ShipEditGui<'a> {
    model_store: &'a ModelStore,
    
    inventory: ModuleInventory,

    action: Option<ShipEditAction>,
    
    ship_offset: Vec2f,
    selected_category: usize,
    selected_model: Option<usize>,
}

impl<'a> ShipEditGui<'a> {
    pub fn new(model_store: &'a ModelStore, inventory: ModuleInventory) -> ShipEditGui<'a> {
        ShipEditGui {
            model_store: model_store,
            
            inventory: inventory,
        
            action: None,
            
            ship_offset: Vec2 { x: -575.0, y: 100.0 },
            selected_category: 0,
            selected_model: None,
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: Vec2f, ship: &ShipStored) -> Option<ShipEditAction> {
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
        
        e.release(|button| {
            match button {
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => {
                            self.on_mouse_left_released(mouse_pos, button, ship);
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

    fn on_mouse_left_pressed(&mut self, mouse_pos: Vec2f, button: mouse::MouseButton) {
        let (_, ref modules) = self.inventory[self.selected_category];
        
        for (i, &(model, count)) in modules.iter().enumerate() {
            let module_offset = Vec2::new(10.0 + (i as f64 * 50.0), 59.0);
            
            if mouse_pos.x >= module_offset.x &&
               mouse_pos.x <= module_offset.x + 48.0 &&
               mouse_pos.y >= module_offset.y &&
               mouse_pos.y <= module_offset.y + 48.0 {
                self.selected_model = Some(i);
            }
        }
    }
    
    fn on_mouse_left_released(&mut self, mouse_pos: Vec2f, button: mouse::MouseButton, ship: &ShipStored) {
        if let Some(selected_model) = self.selected_model {
            let pos_on_ship = self.get_pos_on_ship(mouse_pos);
            
            if ship.is_space_free(pos_on_ship.x as u8, pos_on_ship.y as u8, 1, 1) &&
               (pos_on_ship.x as u8) < 10 && (pos_on_ship.y as u8) < 8 {
                let (_, ref models) = self.inventory[self.selected_category];
                let (selected_model, _) = models[selected_model];
               
                self.action = Some(ShipEditAction::Place(selected_model, pos_on_ship.x as u8, pos_on_ship.y as u8));
            }
        
            self.selected_model = None;
        }
    }

    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache, mouse_pos: Vec2f, ship: &ShipStored) {
        use quack::Set;
        use graphics::*;
        use graphics::text::Text;
        
        // Render background window
        Rectangle::new([0.2, 0.05, 0.3, 0.8])
            .draw([0.0, 0.0, 400.0, 450.0], &context.draw_state, context.transform, gl);
        
        // Label text
        {
            let context = context.trans(5.0, 30.0);
            Text::colored([1.0; 4], 25).draw(
                "module inventory",
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
            
            // Draw the inventory
            let context = context.trans(0.0, 5.0);
            for (cat_num, &(ref category, ref modules)) in self.inventory.iter().enumerate() {
                {
                    let context = context.trans(cat_num as f64 * 77.0, 0.0);
                
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
                }
                
                // Draw module icons
                if cat_num == self.selected_category {
                    let context = context.trans(5.0, 24.0);
                    
                    for (i, &(model, count)) in modules.iter().enumerate() {
                        let context = context.trans(i as f64 * 50.0, 0.0);
                        Rectangle::new([1.0, 0.0, 0.0, 1.0])
                            .draw([0.0, 0.0, 48.0, 48.0], &context.draw_state, context.transform, gl);
                    }
                }
            }
        }
        
        // Draw the selected module
        if let Some(selected_model) = self.selected_model {
            let pos_on_ship = self.get_pos_on_ship(mouse_pos);
            
            if ship.is_space_free(pos_on_ship.x as u8, pos_on_ship.y as u8, 1, 1) &&
               (pos_on_ship.x as u8) < 10 && (pos_on_ship.y as u8) < 8 {
                let render_pos = pos_on_ship*48.0 + self.ship_offset;
               
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([render_pos.x, render_pos.y, 48.0, 48.0], &context.draw_state, context.transform, gl);
            } else {
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([mouse_pos.x - 24.0, mouse_pos.y - 24.0, 48.0, 48.0], &context.draw_state, context.transform, gl);
            }
        }
    }
    
    fn get_pos_on_ship(&self, pos: Vec2f) -> Vec2f {
        ((pos - self.ship_offset) / 48.0).floor()
    }
}