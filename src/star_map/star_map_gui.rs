use ggez::{
    Context, GameResult,
    event::{Event, Keycode, MouseButton},
    graphics::{self, DrawParam, DrawMode, FontId, Point2, Rect, Scale, TextCached},
};

use client_context::ReforgeClientContext;
use gui::TextButton;
use sector_data::{SectorData, SectorId};
use vec::{Vec2, Vec2f};

pub enum StarMapGuiAction {
    Jump(SectorId),
    Close,
}

pub struct StarMapGui {
    selected_sector: Option<SectorId>,
    
    title_lbl: TextCached,
    close_button: TextButton,
    jump_button: TextButton,
}

impl StarMapGui {
    pub fn new(gtx: &ReforgeClientContext) -> GameResult<StarMapGui> {
        Ok(StarMapGui {
            selected_sector: None,
            
            title_lbl: TextCached::new(("star map", gtx.font, Scale::uniform(20.0)))?,
            close_button: TextButton::new(
                gtx.font, "Close", 20.0, [450.0, 400.0], [150.0, 40.0])?,
            jump_button: TextButton::new(
                gtx.font, "Jump", 20.0, [610.0, 400.0], [150.0, 40.0])?,
        })
    }

    pub fn event(&mut self, gtx: &ReforgeClientContext, e: &Event, mouse_pos: Vec2f) -> Option<StarMapGuiAction> {
        use Event::*;

        match *e {
            MouseButtonDown { mouse_btn, .. } => {
                let mouse_pos = mouse_pos - Vec2::new(5.0, 25.0);
            
                for sector in &gtx.sectors {
                    let radius = 10.0;
                    let map_pos = sector.map_position;
                
                    if (map_pos - mouse_pos).length() <= radius {
                        self.selected_sector = Some(sector.id);
                    }
                }
            }
            _ => { }
        }
        
        // Handle buttons
        self.jump_button.event(e, mouse_pos);
        self.close_button.event(e, mouse_pos);
        
        if self.close_button.get_clicked() {
            return Some(StarMapGuiAction::Close);
        }
        
        if self.jump_button.get_clicked() {
            if let Some(selected_sector) = self.selected_sector {
                return Some(StarMapGuiAction::Jump(selected_sector));
            }
        }

        None
    }

    pub fn draw(&mut self, gtx: &ReforgeClientContext, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, [0.2, 0.05, 0.3, 0.8].into())?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.0, 0.0, 800.0, 450.0));
        
        // Render actual star map
        graphics::push_transform(ctx, Some(graphics::get_transform(ctx) * DrawParam {
            dest: Point2::new(5.0, 25.0), ..Default::default()
        }.into_matrix()));
        graphics::apply_transformations(ctx);

        graphics::set_color(ctx, [0.0, 0.0, 0.0, 1.0].into())?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.0, 0.0, 800.0 - 10.0, 400.0 - 30.0));
        
        for sector in &gtx.sectors {
            let radius = 10.0;
            let ref map_pos = sector.map_position;
            
            match self.selected_sector {
                Some(selected_sector) if selected_sector == sector.id => {
                    graphics::set_color(ctx, [0.0, 1.0, 0.0, 1.0].into())?;
                },
                _ => {
                    graphics::set_color(ctx, [0.0, 0.0, 1.0, 1.0].into())?;
                },
            };
        
            graphics::circle(
                ctx, DrawMode::Fill,
                Point2::new((map_pos.x - radius) as f32, (map_pos.y - radius) as f32),
                radius as f32, 4.0)?;
        }
        graphics::set_color(ctx, [1.0; 4].into())?;

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx);
        
        graphics::draw_ex(ctx, &self.title_lbl, DrawParam {
            dest: Point2::new(5.0, 20.0),
            color: Some([1.0; 4].into()),
            ..Default::default()
        })?;
        
        // Draw the buttons
        self.close_button.draw(ctx)?;
        self.jump_button.draw(ctx)?;

        Ok(())
    }
}
