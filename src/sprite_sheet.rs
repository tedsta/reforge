use std::collections::RingBuf;
use std::rc::Rc;
use std::num::Float;
use std::ops::{Deref};

use graphics::{Context, ImageSize};
use opengl_graphics::{Gl, Texture};

use assets::{TextureId, SpriteInfo};

pub enum SpriteAnimation {
    PlayOnce(f64, f64, u32, u32),
    Loop(f64, f64, u32, u32, f64),
    Stay(f64, f64, u32),
}

pub struct SpriteSheet {
    // Texture
    texture: Rc<Texture>,
    
    // Sprite sheet info
    columns: u8,
    frame_width: u32,
    frame_height: u32,
    
    // Sprite sheet state
    current_frame: u32,
    
    // Time stuff
    animations: RingBuf<SpriteAnimation>,
    
    // Whether or not to center the texture
    pub centered: bool,
}

impl SpriteSheet {
    pub fn new(sprite_info: &SpriteInfo) -> SpriteSheet {
        let (texture_width, texture_height) = sprite_info.texture.get_size();
        
        let columns = sprite_info.columns;
        let rows = sprite_info.rows;
        
        SpriteSheet {
            texture: sprite_info.texture.clone(),
            columns: columns,
            frame_width: texture_width/(columns as u32),
            frame_height: texture_height/(rows as u32),
            current_frame: 0,
            animations: RingBuf::new(),
            centered: false,
        }
    }
    
    pub fn add_animation(&mut self, animation: SpriteAnimation) {
        self.animations.push_back(animation);
    }
    
    pub fn draw(&mut self, context: &Context, gl: &mut Gl, x: f64, y: f64, rotation: f64, time: f64) {
        let mut anim_done = false;
        match self.animations.front() {
            Some(animation) =>
                match *animation {
                    SpriteAnimation::PlayOnce(start_time, end_time, start_frame, end_frame) => {
                        if time >= start_time {
                            if time <= end_time {
                                let frame = (((time-start_time)/(end_time-start_time) * ((end_frame - start_frame) as f64)).floor() as u32) + start_frame;
                                self.current_frame = frame;
                            } else {
                                anim_done = true;
                            }
                            self.draw_current_frame(context, gl, x, y, rotation);
                        }
                    },
                    SpriteAnimation::Loop(start_time, end_time, start_frame, end_frame, interval) => {
                        if time >= start_time {
                            if time <= end_time {
                                let mut frame = ((time-start_time) / interval).floor() as u32;
                                frame = frame % (end_frame - start_frame + 1);
                                frame += start_frame;
                                self.current_frame = frame;
                            } else {
                                anim_done = true;
                            }
                            self.draw_current_frame(context, gl, x, y, rotation);
                        }
                    },
                    SpriteAnimation::Stay(start_time, end_time, frame) => {
                        if time >= start_time {
                            if time <= end_time {
                                self.current_frame = frame;
                            } else {
                                anim_done = true;
                            }
                            self.draw_current_frame(context, gl, x, y, rotation);
                        }
                    },
                },
            None => {}
        }
        
        if anim_done {
            self.animations.pop_front();
        }
    }
    
    fn draw_current_frame(&self, context: &Context, gl: &mut Gl, x: f64, y: f64, rotation: f64) {
        use quack::Set;
        use quack::Get;
        use graphics::*;
        use vecmath::row_mat2x3_mul;
    
        let source_x = ((self.current_frame % (self.columns as u32)) as f64) * (self.frame_width as f64);
        let source_y = ((self.current_frame / (self.columns as u32)) as f64) * (self.frame_height as f64);
        
        let half_frame_x = (self.frame_width / 2) as f64;
        let half_frame_y = (self.frame_height / 2) as f64;
        
        let (offset_x, offset_y) =
            if self.centered {
                (half_frame_x, half_frame_y)
            } else {
                (0.0, 0.0)
            };
        
        let rotation_matrix = row_mat2x3_mul(vecmath::rotate_radians(rotation), vecmath::translate([-offset_x, -offset_y]));
        
        let context = context.trans(x, y);
        let context::Transform(transform) = context.get();
        let context = context.set(context::Transform(row_mat2x3_mul(transform, rotation_matrix)));

        Image::new()
            .set(SrcRect([source_x as i32, source_y as i32, self.frame_width as i32, self.frame_height as i32]))
            .draw(self.texture.deref(), &context, gl);
    }
    
    pub fn set_frame(&mut self, frame: u32) {
        self.current_frame = frame;
    }
}
