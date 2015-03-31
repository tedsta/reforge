use std::rc::Rc;

use graphics::Context;
use opengl_graphics::{Gl, Texture};

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
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64) {
        if time >= self.start_time && time <= self.end_time {
            let interp = (time-self.start_time)/(self.end_time-self.start_time);
            let pos = self.start_pos + (self.end_pos-self.start_pos)*interp;
            let rot = self.start_rot + (self.start_rot-self.end_rot)*interp;
            self.sprite_sheet.draw(context, gl, pos.x, pos.y, rot, time);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Basic linear interpolation sim visual
pub struct BeamExitVisual {
    pub start_time: f64,
    pub end_time: f64,
    
    pub beam_start: Vec2f,
    
    pub texture: Rc<Texture>,
}

impl SimVisual for BeamExitVisual {
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64) {
        use graphics::Image;
        use graphics::ImageSize;
        use graphics::Rect;
        use quack::Set;
        use std::ops::Deref;
    
        if time >= self.start_time && time <= self.end_time {
            let (_, height) = self.texture.get_size();
        
            Image::new().set(Rect([self.beam_start.x, self.beam_start.y, 1500.0, height as f64]))
                .draw(self.texture.deref(), &context.draw_state, context.transform, gl);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Basic linear interpolation sim visual
pub struct BeamVisual {
    pub start_time: f64,
    pub end_time: f64,
    
    pub beam_start: Vec2f,
    pub beam_end: Vec2f,
    
    pub part: Rc<Texture>,
    pub end: SpriteSheet,
}

impl BeamVisual {
    pub fn new(start_time: f64, end_time: f64, beam_start: Vec2f, beam_end: Vec2f, part: Rc<Texture>, mut end: SpriteSheet) -> BeamVisual {
        end.centered = true;
    
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
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64) {
        use graphics::Image;
        use graphics::ImageSize;
        use graphics::Rect;
        use quack::Set;
        use std::ops::Deref;
        use std::num::Float;
    
        if time >= self.start_time && time <= self.end_time {
            let interp = (time-self.start_time)/(self.end_time-self.start_time);
            
            let beam_pos = self.beam_start + (self.beam_end - self.beam_start)*interp;
            
            let (width, height) = self.end.get_frame_size();
            let (width, height) = (width as f64, height as f64);
        
            // Draw beam part
            Image::new().set(Rect([beam_pos.x, beam_pos.y - (height/2.0), 1500.0, height as f64]))
                .draw(self.part.deref(), &context.draw_state, context.transform, gl);
                
            // Draw beam end
            self.end.draw(&context, gl, beam_pos.x, beam_pos.y, (180.0).to_radians(), time - self.start_time);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Sprite sheet sim visual
pub struct SpriteVisual {
    pub position: Vec2f,
    pub sprite_sheet: SpriteSheet,
}

impl SimVisual for SpriteVisual {
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64) {
        self.sprite_sheet.draw(context, gl, self.position.x, self.position.y, 0.0, time);
    }
}