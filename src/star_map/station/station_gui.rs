use event::{Events, GenericEvent, RenderArgs};
use graphics::{Context, Rectangle};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use asset_store::AssetStore;
use gui::TextButton;
use module::{IModule, Module, ModuleIndex};
use net::ClientId;
use sector_data::SectorData;
use ship::ShipStored;
use sim::SimEffects;
use star_map::{StarMapGuiAction, StarMapGui};

use super::StationAction;
use super::ShipEditGui;

pub struct StationGui {
    mouse_x: f64,
    mouse_y: f64,
    
    // Ship editor stuff
    ship_edit_gui: ShipEditGui,
    
    // Star map stuff
    star_map_button: TextButton,
    star_map_gui: StarMapGui,
    show_star_map: bool,
    
    // Logout button
    logout_button: TextButton,
}

impl StationGui {
    pub fn new(sectors: Vec<SectorData>) -> StationGui {    
        StationGui {
            mouse_x: 0.0,
            mouse_y: 0.0,
            
            ship_edit_gui: ShipEditGui::new(),
            
            star_map_button: TextButton::new("star map".to_string(), 20, [550.0, 50.0], [120.0, 40.0]),
            star_map_gui: StarMapGui::new(sectors),
            show_star_map: false,
            
            logout_button: TextButton::new("logout".to_string(), 20, [550.0, 100.0], [120.0, 40.0]),
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, client_ship: &Option<ShipStored>) -> Option<StationAction> {
        use event::*;
        
        e.mouse_cursor(|x, y| {
            self.mouse_x = x;
            self.mouse_y = y;
        });
        
        self.star_map_button.event(e, [self.mouse_x, self.mouse_y]);
        if self.star_map_button.get_clicked() {
            self.show_star_map = true;
        }
        
        if self.show_star_map {
            if let Some(star_map_action) = self.star_map_gui.event(e, [self.mouse_x - 200.0, self.mouse_y - 200.0]) {
                match star_map_action {
                    StarMapGuiAction::Jump(sector) => {
                        self.show_star_map = false;
                        return Some(StationAction::Jump(sector));
                    },
                    StarMapGuiAction::Close => {
                        self.show_star_map = false;
                    },
                }
            }
            
            return None;
        }
        
        self.ship_edit_gui.event(e, [self.mouse_x - 200.0, self.mouse_y - 200.0]);
        
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                Button::Mouse(button) => {
                    let (mouse_x, mouse_y) = (self.mouse_x, self.mouse_y);
                    match button {
                        mouse::MouseButton::Left => self.on_mouse_left_pressed(mouse_x, mouse_y, client_ship),
                        mouse::MouseButton::Right => self.on_mouse_right_pressed(mouse_x, mouse_y, client_ship),
                        _ => {},
                    }
                },
            }
        });
        
        self.logout_button.event(e, [self.mouse_x, self.mouse_y]);
        if self.logout_button.get_clicked() {
            // TODO: Logout
        }
        
        None
    }
    
    pub fn draw(
        &mut self,
        context: &Context,
        gl: &mut Gl,
        glyph_cache: &mut GlyphCache,
        asset_store: &AssetStore,
        sim_effects: &mut SimEffects,
        client_ship: &Option<ShipStored>,
        time: f64,
        dt: f64,
    )
    {
        use graphics::*;
        
        // Clear the screen
        clear([0.0; 4], gl);
        
        // Draw player's ship
        if let &Some(ref client_ship) = client_ship {
            let ref context = context.trans(300.0, 300.0);
            sim_effects.update(context, gl, client_ship.id, time);
        }
        
        self.ship_edit_gui.draw(&context.trans(875.0, 200.0), gl, glyph_cache);
        
        self.star_map_button.draw(context, gl, glyph_cache);
        self.logout_button.draw(context, gl, glyph_cache);
        
        if self.show_star_map {
            self.star_map_gui.draw(&context.trans(200.0, 200.0), gl, glyph_cache);
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    fn on_mouse_left_pressed(&mut self, x: f64, y: f64, client_ship: &Option<ShipStored>) {
    }
    
    fn on_mouse_right_pressed(&mut self, x: f64, y: f64, client_ship: &Option<ShipStored>) {
    }
}
