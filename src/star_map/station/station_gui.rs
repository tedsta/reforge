use ggez::{
    Context, GameResult,
    event::{Event, Keycode, MouseButton},
    graphics::{self, DrawParam, DrawMode, FontId, Point2, Rect},
};

use asset_store::AssetStore;
//use chat::{ChatGui, ChatGuiAction};
use client_context::ReforgeClientContext;
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

pub struct StationGui {
    mouse_pos: Vec2f,

    // Ship editor stuff
    ship_edit_gui: ShipEditGui,
    
    // Chat
    //chat_gui_pos: Vec2f,
    //pub chat_gui: &'a mut ChatGui,
    
    // Star map stuff
    star_map_button: TextButton,
    star_map_gui: StarMapGui,
    show_star_map: bool,
    
    // Logout button
    logout_button: TextButton,
}

impl StationGui {
    pub fn new(
        gtx: &ReforgeClientContext,
        //chat_gui: &'a mut ChatGui,
        module_inventory: ModuleInventory) -> GameResult<StationGui>
    {
        Ok(StationGui {
            mouse_pos: Vec2::new(0.0, 0.0),

            ship_edit_gui: ShipEditGui::new(gtx, module_inventory)?,
            
            //chat_gui_pos: Vec2::new(5.0, 720.0 - 200.0 - 5.0),
            //chat_gui: chat_gui,
            
            star_map_button: TextButton::new(
                gtx.font, "star map", 20.0, [550.0, 50.0], [120.0, 40.0])?,
            star_map_gui: StarMapGui::new(gtx)?,
            show_star_map: false,
            
            logout_button: TextButton::new(gtx.font, "logout", 20.0, [550.0, 100.0], [120.0, 40.0])?,
        })
    }
    
    pub fn event(&mut self, gtx: &ReforgeClientContext, e: &Event, client_ship: &Option<ShipStored>) -> Option<StationAction> {
        use Event::*;
        match *e {
            MouseMotion { x, y, .. } => {
                self.mouse_pos = Vec2::new(x as f64, y as f64);
            },
            _ => { },
        }
        
        self.star_map_button.event(e, self.mouse_pos);
        if self.star_map_button.get_clicked() {
            self.show_star_map = true;
        }
        
        if self.show_star_map {
            if let Some(star_map_action) = self.star_map_gui.event(
                gtx, e, self.mouse_pos - Vec2::new(200.0, 200.0)
            ) {
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
        
        /*if let Some(chat_action) = self.chat_gui.event(e, self.mouse_pos - self.chat_gui_pos) {
            match chat_action {
                ChatGuiAction::SendMsg(msg) => {
                    return Some(StationAction::Chat(msg));
                },
            }
        }*/
        
        if let Some(ship_edit) = self.ship_edit_gui.event(
            gtx, e, self.mouse_pos - Vec2::new(800.0, 200.0), client_ship.as_ref().unwrap())
        {
            return Some(StationAction::ShipEdit(ship_edit));
        }
        
        self.logout_button.event(e, self.mouse_pos);
        if self.logout_button.get_clicked() {
            return Some(StationAction::Logout);
        }
        
        None
    }
    
    pub fn draw(
        &mut self,
        gtx: &ReforgeClientContext,
        ctx: &mut Context,
        sim_effects: &mut SimEffects,
        client_ship: &Option<ShipStored>,
        time: f64,
        dt: f64) -> GameResult<()>
    {
        // Draw placeable area for ship editor
        if self.ship_edit_gui.selected_model.is_some() {
            graphics::set_color(ctx, [0.0, 1.0, 0.0, 0.5].into())?;
            graphics::rectangle(ctx, DrawMode::Fill, Rect::new(
                300.0, 300.0, 10.0 * 48.0, 8.0 * 48.0))?;
        }
        
        // Draw player's ship
        if let &Some(ref client_ship) = client_ship {
            graphics::push_transform(ctx, Some(graphics::get_transform(ctx) * DrawParam {
                dest: Point2::new(300.0, 300.0), ..Default::default()
            }.into_matrix()));
            graphics::apply_transformations(ctx);

            sim_effects.update(ctx, client_ship.id, time)?;

            graphics::pop_transform(ctx);
            graphics::apply_transformations(ctx);
        }
        
        graphics::push_transform(ctx, Some(graphics::get_transform(ctx) * DrawParam {
            dest: Point2::new(800.0, 200.0), ..Default::default()
        }.into_matrix()));
        graphics::apply_transformations(ctx);

        self.ship_edit_gui.draw(
            gtx, ctx, self.mouse_pos - Vec2::new(800.0, 200.0),
            client_ship.as_ref().unwrap())?;

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx);
        
        self.star_map_button.draw(ctx)?;
        self.logout_button.draw(ctx)?;
        
        //self.chat_gui.draw(&context.trans(self.chat_gui_pos.x, self.chat_gui_pos.y), gl, glyph_cache);
        
        if self.show_star_map {
            graphics::push_transform(ctx, Some(graphics::get_transform(ctx) * DrawParam {
                dest: Point2::new(200.0, 200.0), ..Default::default()
            }.into_matrix()));
            graphics::apply_transformations(ctx);

            self.star_map_gui.draw(gtx, ctx)?;

            graphics::pop_transform(ctx);
            graphics::apply_transformations(ctx);
        }

        Ok(())
    }
}
