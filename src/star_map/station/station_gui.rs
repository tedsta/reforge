use event::{Events, GenericEvent, RenderArgs};
use graphics::{Context, Rectangle};
use input::{keyboard, mouse, Button};
use opengl_graphics::{GlGraphics, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use asset_store::AssetStore;
use chat::{ChatGui, ChatGuiAction};
use gui::TextButton;
use module::{IModule, ModelStore, Module, ModuleIndex};
use net::ClientId;
use sector_data::SectorData;
use ship::ShipStored;
use sim::SimEffects;
use star_map::{StarMapGuiAction, StarMapGui};
use vec::{Vec2, Vec2f};

use super::StationAction;
use super::ship_edit_gui::{ModuleInventory, ShipEditGui};

pub struct StationGui<'a> {
    mouse_pos: Vec2f,
    
    // Ship editor stuff
    ship_edit_gui: ShipEditGui<'a>,
    
    // Chat
    chat_gui_pos: Vec2f,
    pub chat_gui: &'a mut ChatGui,
    
    // Star map stuff
    star_map_button: TextButton,
    star_map_gui: StarMapGui,
    show_star_map: bool,
    
    // Logout button
    logout_button: TextButton,
}

impl<'a> StationGui<'a> {
    pub fn new(model_store: &'a ModelStore,
               chat_gui: &'a mut ChatGui,
               sectors: Vec<SectorData>,
               module_inventory: ModuleInventory) -> StationGui<'a> {
        StationGui {
            mouse_pos: Vec2 { x: 0.0, y: 0.0 },
            
            ship_edit_gui: ShipEditGui::new(model_store, module_inventory),
            
            chat_gui_pos: Vec2::new(5.0, 720.0 - 200.0 - 5.0),
            chat_gui: chat_gui,
            
            star_map_button: TextButton::new("star map".to_string(), 24, [550.0, 50.0], [120.0, 40.0]),
            star_map_gui: StarMapGui::new(sectors),
            show_star_map: false,
            
            logout_button: TextButton::new("logout".to_string(), 24, [550.0, 100.0], [120.0, 40.0]),
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, client_ship: &Option<ShipStored>) -> Option<StationAction> {
        use event::*;
        
        e.mouse_cursor(|x, y| {
            self.mouse_pos.x = x;
            self.mouse_pos.y = y;
        });
        
        self.star_map_button.event(e, [self.mouse_pos.x, self.mouse_pos.y]);
        if self.star_map_button.get_clicked() {
            self.show_star_map = true;
        }
        
        if self.show_star_map {
            if let Some(star_map_action) = self.star_map_gui.event(e, [self.mouse_pos.x - 200.0, self.mouse_pos.y - 200.0]) {
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
        
        if let Some(chat_action) = self.chat_gui.event(e, self.mouse_pos - self.chat_gui_pos) {
            match chat_action {
                ChatGuiAction::SendMsg(msg) => {
                    return Some(StationAction::Chat(msg));
                },
            }
        }
        
        if let Some(ship_edit) = self.ship_edit_gui.event(e, self.mouse_pos - Vec2::new(800.0, 200.0), client_ship.as_ref().unwrap()) {
            return Some(StationAction::ShipEdit(ship_edit));
        }
        
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => self.on_mouse_left_pressed(client_ship),
                        mouse::MouseButton::Right => self.on_mouse_right_pressed(client_ship),
                        _ => {},
                    }
                },
            }
        });
        
        self.logout_button.event(e, [self.mouse_pos.x, self.mouse_pos.y]);
        if self.logout_button.get_clicked() {
            return Some(StationAction::Logout);
        }
        
        None
    }
    
    pub fn draw(
        &mut self,
        context: &Context,
        gl: &mut GlGraphics,
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
        
        // Draw placeable area for ship editor
        if self.ship_edit_gui.selected_model.is_some() {
            Rectangle::new([0.0, 1.0, 0.0, 0.5])
                .draw([300.0, 300.0, 10.0 * 48.0, 8.0 * 48.0], &context.draw_state, context.transform, gl);
        }
        
        // Draw player's ship
        if let &Some(ref client_ship) = client_ship {
            let ref context = context.trans(300.0, 300.0);
            sim_effects.update(context, gl, client_ship.id, time);
        }
        
        self.ship_edit_gui.draw(&context.trans(800.0, 200.0),
                                gl,
                                glyph_cache,
                                self.mouse_pos - Vec2::new(800.0, 200.0),
                                client_ship.as_ref().unwrap());
        
        self.star_map_button.draw(context, gl, glyph_cache);
        self.logout_button.draw(context, gl, glyph_cache);
        
        self.chat_gui.draw(&context.trans(self.chat_gui_pos.x, self.chat_gui_pos.y), gl, glyph_cache);
        
        if self.show_star_map {
            self.star_map_gui.draw(&context.trans(200.0, 200.0), gl, glyph_cache);
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    fn on_mouse_left_pressed(&mut self, client_ship: &Option<ShipStored>) {
    }
    
    fn on_mouse_right_pressed(&mut self, client_ship: &Option<ShipStored>) {
    }
}
