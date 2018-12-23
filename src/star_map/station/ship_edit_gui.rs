use ggez::{
    Context, GameResult,
    event::{Event, Keycode, MouseButton},
    graphics::{self, DrawParam, DrawMode, FontId, Point2, Rect, Scale, TextCached},
};

use client_context::ReforgeClientContext;
use module::{ModelIndex, ModelStore};
use ship::ShipStored;
use util::with_translate;
use vec::{Vec2, Vec2f};

use super::ShipEditAction;

pub type ModuleInventory = Vec<(String, Vec<(ModelIndex, u16)>)>;

pub struct ShipEditGui {
    inventory: ModuleInventory,

    ship_offset: Vec2f,
    selected_category: usize,
    pub selected_model: Option<usize>,

    inventory_lbl: TextCached,
    inventory_category_lbls: Vec<TextCached>,
}

impl ShipEditGui {
    pub fn new(
        gtx: &ReforgeClientContext,
        inventory: ModuleInventory)
        -> GameResult<ShipEditGui>
    {
        let mut inventory_category_lbls = Vec::new();
        for &(ref category, ref modules) in &inventory {
            inventory_category_lbls.push(TextCached::new(
                (category.as_ref(), gtx.font, Scale::uniform(24.0)))?);
        }

        Ok(ShipEditGui {
            inventory: inventory,
        
            ship_offset: Vec2 { x: -500.0, y: 100.0 },
            selected_category: 0,
            selected_model: None,

            inventory_lbl: TextCached::new(("module inventory", gtx.font, Scale::uniform(16.0)))?,
            inventory_category_lbls,
        })
    }

    pub fn event(&mut self, gtx: &ReforgeClientContext, e: &Event, mouse_pos: Vec2f, ship: &ShipStored) -> Option<ShipEditAction> {
        use Event::*;
        match *e {
            MouseButtonDown { mouse_btn, x, y, .. } => {
                match mouse_btn {
                    MouseButton::Left => {
                        self.on_mouse_left_pressed(gtx, mouse_pos)
                    },
                    _ => { None },
                }
            }
            MouseButtonUp { mouse_btn, x, y, .. } => {
                match mouse_btn {
                    MouseButton::Left => {
                        self.on_mouse_left_released(gtx, mouse_pos, ship)
                    },
                    _ => { None },
                }
            },
            _ => { None }
        }
    }

    fn on_mouse_left_pressed(&mut self, gtx: &ReforgeClientContext, mouse_pos: Vec2f) -> Option<ShipEditAction> {
        for i in (0 .. self.inventory.len()) {
            let category_offset = Vec2::new(5.0 + (i as f64 * 60.0), 35.0);
            let label_width = 58.0;
            let label_height = 24.0;
            
            if mouse_pos.x >= category_offset.x &&
               mouse_pos.x <= category_offset.x + label_width &&
               mouse_pos.y >= category_offset.y &&
               mouse_pos.y <= category_offset.y + label_height {
                self.selected_category = i;
            }
        }
    
        let (_, ref modules) = self.inventory[self.selected_category];
        
        for (i, &(model_index, count)) in modules.iter().enumerate() {
            let model = model_index.get(&gtx.model_store);
        
            let module_offset = Vec2::new(10.0 + (i as f64 * 50.0), 59.0);
            
            if mouse_pos.x >= module_offset.x &&
               mouse_pos.x <= module_offset.x + (model.shape.side() as f64 * 48.0) &&
               mouse_pos.y >= module_offset.y &&
               mouse_pos.y <= module_offset.y + (model.shape.side() as f64 * 48.0) {
                self.selected_model = Some(i);
            }
        }

        None
    }
    
    fn on_mouse_left_released(&mut self, gtx: &ReforgeClientContext, mouse_pos: Vec2f, ship: &ShipStored) -> Option<ShipEditAction> {
        if let Some(selected_model) = self.selected_model {
            let pos_on_ship = self.get_pos_on_ship(mouse_pos);
            
            let (_, ref models) = self.inventory[self.selected_category];
            let (model_index, _) = models[selected_model];
            let model = model_index.get(&gtx.model_store);
            
            if ship.is_space_free(pos_on_ship.x as u8, pos_on_ship.y as u8, &model.shape) &&
               (pos_on_ship.x as u8) < 10 && (pos_on_ship.y as u8) < 8 {
                return Some(ShipEditAction::Place(model_index, pos_on_ship.x as u8, pos_on_ship.y as u8));
            }
        
            self.selected_model = None;
        }

        None
    }

    pub fn draw(&mut self, gtx: &ReforgeClientContext, ctx: &mut Context, mouse_pos: Vec2f, ship: &ShipStored) -> GameResult<()> {
        // Render background window
        graphics::set_color(ctx, [0.2, 0.05, 0.3, 0.8].into())?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.0, 0.0, 475.0, 450.0));
        
        // Label text
        graphics::set_color(ctx, [1.0; 4].into())?;
        graphics::draw_ex(
            ctx, &self.inventory_lbl, Default::default())?;
        
        with_translate(ctx, Point2::new(5.0, 35.0), |ctx| -> GameResult<()> {
            // Draw the inventory
            let category_iter =
                self.inventory.iter().zip(self.inventory_category_lbls.iter()).enumerate();
            for (cat_num, (&(ref category, ref modules), category_lbl)) in category_iter {
                with_translate(ctx, Point2::new(cat_num as f32 * 60.0, 0.0), |ctx| -> GameResult<()> {
                    if cat_num == self.selected_category {
                        graphics::set_color(ctx, [1.0, 0.0, 0.0, 1.0].into())?;
                    } else {
                        graphics::set_color(ctx, [0.0, 1.0, 0.0, 1.0].into())?;
                    };
                    graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.0, 0.0, 58.0, 24.0));
                    
                    // Category label
                    graphics::set_color(ctx, [1.0; 4].into())?;
                    graphics::draw_ex(ctx, category_lbl, DrawParam {
                        dest: Point2::new(5.0, 21.0), ..Default::default()
                    })?;

                    Ok(())
                })?;
                
                // Draw module icons
                if cat_num == self.selected_category {
                    for (i, &(model_index, count)) in modules.iter().enumerate() {
                        let model = model_index.get(&gtx.model_store);
                        graphics::draw(
                            ctx, &model.icon,
                            Point2::new(5.0 + i as f32 * 50.0, 30.0),
                            0.0);
                    }
                }
            }

            Ok(())
        })?;
        
        // Draw the selected module
        if let Some(selected_model) = self.selected_model {
            let (_, ref models) = self.inventory[self.selected_category];
            let (model_index, _) = models[selected_model];
            let model = model_index.get(&gtx.model_store);
        
            let pos_on_ship = self.get_pos_on_ship(mouse_pos);
            
            if ship.is_space_free(pos_on_ship.x as u8, pos_on_ship.y as u8, &model.shape) &&
               (pos_on_ship.x as u8) < 10 && (pos_on_ship.y as u8) < 8 {
                let render_pos = pos_on_ship * 48.0 + self.ship_offset;
                graphics::draw_ex(
                    ctx, &model.icon,
                    DrawParam {
                        dest: Point2::new(render_pos.x as f32, render_pos.y as f32),
                        ..Default::default()
                    })?;
            } else {
                graphics::draw_ex(
                    ctx, &model.icon,
                    DrawParam {
                        dest: Point2::new(
                            (mouse_pos.x - (48.0 / 2.0)) as f32,
                            (mouse_pos.y - (48.0 / 2.0)) as f32),
                        ..Default::default()
                    })?;
            }
        }

        Ok(())
    }
    
    fn get_pos_on_ship(&self, pos: Vec2f) -> Vec2f {
        ((pos - self.ship_offset) / 48.0).floor()
    }
}
