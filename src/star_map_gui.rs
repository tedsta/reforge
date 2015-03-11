use std::cell::RefCell;

use sdl2_window::Sdl2Window;
use event::{Events, GenericEvent};
use graphics::{Context};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use gui::TextButton;
use sector_data::{SectorData, SectorId};
use vec::Vec2;

pub enum StarMapAction {
    Jump(SectorId),
    Close,
}

pub struct StarMapGui {
    sectors: Vec<SectorData>,

    action: Option<StarMapAction>,
    
    selected_sector: Option<SectorId>,
    
    // Buttons
    close_button: TextButton,
    jump_button: TextButton,
}

impl StarMapGui {
    pub fn new(sectors: Vec<SectorData>) -> StarMapGui {
        StarMapGui {
            sectors: sectors,
        
            action: None,
            
            selected_sector: None,
            
            close_button: TextButton::new("Close".to_string(), 20, [450.0, 400.0], [150.0, 40.0]),
            jump_button: TextButton::new("Jump".to_string(), 20, [610.0, 400.0], [150.0, 40.0]),
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2]) -> Option<StarMapAction> {
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
        
        // Handle buttons
        self.jump_button.event(e, mouse_pos);
        self.close_button.event(e, mouse_pos);
        
        if self.close_button.get_clicked() {
            self.action = Some(StarMapAction::Close);
        }
        
        if self.jump_button.get_clicked() {
            if let Some(selected_sector) = self.selected_sector {
                self.action = Some(StarMapAction::Jump(selected_sector));
            }
        }
        
        self.action.take()
    }

    fn on_mouse_left_pressed(&mut self, mouse_pos: [f64; 2], button: mouse::MouseButton) {
        let mouse_pos = Vec2 { x: mouse_pos[0] - 5.0, y: mouse_pos[1] - 25.0 };
    
        for sector in &self.sectors {
            let radius = 10.0;
            let map_pos = sector.map_position;
        
            if (map_pos - mouse_pos).length() <= radius {
                self.selected_sector = Some(sector.id);
            }
        }
    }

    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use quack::Set;
        use graphics::*;
        use graphics::text::Text;
        
        Rectangle::new([0.2, 0.05, 0.3, 0.8])
            .draw([0.0, 0.0, 800.0, 450.0], context, gl);
        
        // Render actual star map
        {
            let ref context = context.trans(5.0, 25.0);
        
            Rectangle::new([0.0, 0.0, 0.0, 1.0])
                .draw([0.0, 0.0, 800.0 - 10.0, 400.0 - 30.0], context, gl);
            
            for sector in &self.sectors {
                let radius = 10.0;
                let ref map_pos = sector.map_position;
                
                let sector_circle =
                    match self.selected_sector {
                        Some(selected_sector) if selected_sector == sector.id => Ellipse::new([0.0, 1.0, 0.0, 1.0]),
                        _ => Ellipse::new([0.0, 0.0, 1.0, 1.0]),
                    };
            
                sector_circle.draw([map_pos.x - radius, map_pos.y - radius, radius, radius], &context, gl);
            }
        }
        
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