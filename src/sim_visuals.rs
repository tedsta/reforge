use graphics::Context;
use opengl_graphics::Gl;

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
pub struct BeamVisual {
    pub start_time: f64,
    pub end_time: f64,
    
    pub beam_start: Vec2f,
    pub beam_end: Vec2f,
}

impl SimVisual for BeamVisual {
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64) {
        use graphics::Line;
    
        if time >= self.start_time && time <= self.end_time {
            let interp = (time-self.start_time)/(self.end_time-self.start_time);
            
            let beam_pos = self.beam_start + (self.beam_end - self.beam_start)*interp;
            
            let start_pos = beam_pos + Vec2 { x: 1000.0, y: 0.0 };
            
            Line::new([1.0, 0.0, 0.0, 1.0], 1.0)
                .draw([self.beam_start.x, self.beam_start.y, self.beam_end.x, self.beam_end.y], &context, gl);
            
            Line::new([1.0, 0.0, 0.0, 1.0], 2.0)
                .draw([start_pos.x, start_pos.y, beam_pos.x, beam_pos.y], &context, gl);
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