use graphics::Context;
use opengl_graphics::Gl;

use sim::SimVisual;
use sprite_sheet::SpriteSheet;
use vec::{Vec2, Vec2f};

// Basic linear isizeerpolation sim visual
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
            let isizeerp = (time-self.start_time)/(self.end_time-self.start_time);
            let pos = self.start_pos + (self.end_pos-self.start_pos)*isizeerp;
            let rot = self.start_rot + (self.start_rot-self.end_rot)*isizeerp;
            self.sprite_sheet.draw(context, gl, pos.x, pos.y, rot, time);
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