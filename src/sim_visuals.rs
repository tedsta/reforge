use std::rc::Rc;

use ggez::{Context, GameResult};
use ggez::graphics::{self, DrawParam, Image, Point2, Rect};

use sim::SimVisual;
use sprite_sheet::SpriteSheet;
use vec::{Vec2, Vec2f};

// Basic linear interpolation sim visual
pub struct LerpVisual {
    pub start_time: f64,
    pub end_time: f64,
    pub start_pos: Vec2f,
    pub end_pos: Vec2f,
    pub start_rot: f64,
    pub end_rot: f64,
    pub sprite_sheet: SpriteSheet,
}

impl SimVisual for LerpVisual {
    fn draw(&mut self, ctx: &mut Context, time: f64) -> GameResult<()> {
        if time >= self.start_time && time <= self.end_time {
            let interp = (time-self.start_time)/(self.end_time-self.start_time);
            let pos = self.start_pos + (self.end_pos-self.start_pos)*interp;
            let rot = self.start_rot + (self.end_rot-self.start_rot)*interp;
            self.sprite_sheet.draw(ctx, pos.x, pos.y, rot, time)?;
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Basic linear interpolation sim visual
pub struct BeamExitVisual {
    pub start_time: f64,
    pub end_time: f64,
    
    pub beam_start: Vec2f,
    
    pub texture: Rc<Image>,
}

impl SimVisual for BeamExitVisual {
    fn draw(&mut self, ctx: &mut Context, time: f64) -> GameResult<()> {
        if time >= self.start_time && time <= self.end_time {
            graphics::draw_ex(
                ctx, &*self.texture,
                DrawParam {
                    dest: Point2::new(
                        self.beam_start.x as f32,
                        self.beam_start.y as f32),
                    src: Rect::new(0.0, 0.0, 1500.0, 1.0),
                    ..Default::default()
                })?;
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Basic linear interpolation sim visual
pub struct BeamVisual {
    pub start_time: f64,
    pub end_time: f64,
    
    pub beam_start: Vec2f,
    pub beam_end: Vec2f,
    
    pub part: Rc<Image>,
    pub end: SpriteSheet,
}

impl BeamVisual {
    pub fn new(start_time: f64, end_time: f64, beam_start: Vec2f, beam_end: Vec2f, part: Rc<Image>, mut end: SpriteSheet) -> BeamVisual {
        end.center();
    
        BeamVisual {
            start_time: start_time,
            end_time: end_time,
            
            beam_start: beam_start,
            beam_end: beam_end,
            
            part: part,
            end: end,
        }
    }
}

impl SimVisual for BeamVisual {
    fn draw(&mut self, ctx: &mut Context, time: f64) -> GameResult<()> {
        use float::Radians;
    
        if time >= self.start_time && time <= self.end_time {
            let interp = (time-self.start_time)/(self.end_time-self.start_time);
            
            let beam_pos = self.beam_start + (self.beam_end - self.beam_start)*interp;
            
            let (width, height) = self.end.get_frame_size();
            let (width, height) = (width as f32, height as f32);
        
            // Draw beam part
            graphics::draw_ex(
                ctx, &*self.part,
                DrawParam {
                    dest: Point2::new(
                        beam_pos.x as f32,
                        (beam_pos.y as f32) - height / 2.0),
                    src: Rect::new(0.0, 0.0, 1500.0, 1.0),
                    ..Default::default()
                })?;
                
            // Draw beam end
            self.end.draw(ctx, beam_pos.x, beam_pos.y, Radians::_180(), time - self.start_time)?;
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Sprite sheet sim visual
pub struct SpriteVisual {
    pub position: Vec2f,
    pub rotation: f64,
    pub sprite_sheet: SpriteSheet,
}

impl SpriteVisual {
    pub fn new(position: Vec2f, rotation: f64, sprite_sheet: SpriteSheet) -> SpriteVisual {
        SpriteVisual {
            position: position,
            rotation: rotation,
            sprite_sheet: sprite_sheet,
        }
    }
}

impl SimVisual for SpriteVisual {
    fn draw(&mut self, ctx: &mut Context, time: f64) -> GameResult<()> {
        self.sprite_sheet.draw(ctx, self.position.x, self.position.y, self.rotation, time)
    }
}
