use std::collections::{Deque, RingBuf};

use serialize::{Encodable, Encoder, Decodable, Decoder};

use rsfml::graphics::Texture;

use binary_encode::HasContext;

use asset_store::AssetStore;
use assets::{TextureId, SpriteInfo};
use sfml_renderer::SfmlRenderer;

pub enum SpriteAnimation {
    PlayOnce(f32, f32, u16, u16),
    Loop(f32, f32, u16, u16, f32),
    Stay(f32, f32, u16),
}

pub struct SpriteSheet {
    // Texture
    texture: TextureId,
    
    // Sprite sheet info
    columns: u8,
    rows: u8,
    frame_width: u16,
    frame_height: u16,
    
    // Sprite sheet state
    current_frame: u16,
    start_frame: u16,
    end_frame: u16,
    
    // Time stuff
    animations: RingBuf<SpriteAnimation>,
}

impl SpriteSheet {
    pub fn new(sprite_info: &SpriteInfo) -> SpriteSheet {
        let texture_width = sprite_info.texture_width as u16;
        let texture_height = sprite_info.texture_height as u16;
        
        let columns = sprite_info.columns;
        let rows = sprite_info.rows;
        
        SpriteSheet {
            texture: sprite_info.texture,
            columns: columns,
            rows: rows,
            frame_width: texture_width/(columns as u16),
            frame_height: texture_height/(rows as u16),
            current_frame: 0,
            start_frame: 0,
            end_frame: (columns as u16)*(rows as u16) - 1,
            animations: RingBuf::new(),
        }
    }
    
    pub fn add_animation(&mut self, animation: SpriteAnimation) {
        self.animations.push(animation);
    }
    
    pub fn draw(&mut self, renderer: &SfmlRenderer, x: f32, y: f32, rotation: f32, time: f32) {
        let mut anim_done = false;
        match (&self.animations as &Deque<SpriteAnimation>).front() {
            Some(animation) =>
                match *animation {
                    PlayOnce(start_time, end_time, start_frame, end_frame) => {
                        if time >= start_time {
                            if time <= end_time {
                                let mut frame = ((time-start_time)/(end_time-start_time) * ((end_frame - start_frame) as f32)).floor() as u16;
                                self.current_frame = frame;
                                self.draw_current_frame(renderer, x, y, rotation);
                            } else if end_time != 0.0 {
                                anim_done = true;
                            }
                        }
                    },
                    Loop(start_time, end_time, start_frame, end_frame, interval) => {
                        if time >= start_time {
                            if time <= end_time {
                                let mut frame = ((time-start_time) / interval).floor() as u16;
                                frame = frame % (end_frame - start_frame + 1);
                                frame += start_frame;
                                self.current_frame = frame;
                                self.draw_current_frame(renderer, x, y, rotation);
                            } else if end_time != 0.0 {
                                anim_done = true;
                            }
                        }
                    },
                    Stay(start_time, end_time, frame) => {
                        if time >= start_time {
                            if time <= end_time {
                                self.current_frame = frame;
                                self.draw_current_frame(renderer, x, y, rotation);
                            } else if end_time != 0.0 {
                                anim_done = true;
                            }
                        }
                    },
                },
            None => {}
        }
        
        if anim_done {
            (&mut self.animations as &mut Deque<SpriteAnimation>).pop_front();
        }
    }
    
    fn draw_current_frame(&self, renderer: &SfmlRenderer, x: f32, y: f32, rotation: f32) {
        let source_x = ((self.current_frame % (self.columns as u16)) as f32) * (self.frame_width as f32);
        let source_y = ((self.current_frame / (self.columns as u16)) as f32) * (self.frame_height as f32);
        renderer.draw_texture_source(self.texture, x, y, rotation, source_x, source_y, self.frame_width as f32, self.frame_height as f32);
    }
    
    pub fn set_frame(&mut self, frame: u16) {
        self.current_frame = frame;
    }
    
    pub fn get_texture(&self) -> TextureId {
        self.texture
    }
}