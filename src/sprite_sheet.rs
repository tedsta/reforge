use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use num::Float;

use ggez::{Context, GameResult};
use ggez::graphics::{self, DrawParam, Image, Point2, Rect};

use asset_store::SpriteInfo;
use vec::{Vec2, Vec2f};

pub enum SpriteAnimation {
    PlayOnce(f64, f64, u32, u32),
    Loop(f64, f64, u32, u32, f64),
    Stay(f64, f64, u32),
}

pub struct SpriteSheet {
    // Texture
    texture: Rc<Image>,
    
    // Sprite sheet info
    rows: u32,
    columns: u32,
    frame_width: u32,
    frame_height: u32,
    
    // Sprite sheet state
    current_frame: u32,
    
    // Time stuff
    animations: VecDeque<SpriteAnimation>,
    
    // Animation name -> frames map
    anim_map: HashMap<String, (u32, u32)>,
    
    // Whether or not to center the texture
    pub center: Vec2f,
}

impl SpriteSheet {
    pub fn new(sprite_info: &SpriteInfo) -> SpriteSheet {
        let sheet_width = sprite_info.texture.width();
        let sheet_height = sprite_info.texture.height();
        
        let columns = sprite_info.columns as u32;
        let rows = sprite_info.rows as u32;
        
        SpriteSheet {
            texture: sprite_info.texture.clone(),
            rows: rows,
            columns: columns,
            frame_width: sheet_width / columns,
            frame_height: sheet_height / rows,
            current_frame: 0,
            animations: VecDeque::new(),
            anim_map: sprite_info.animations.clone(),
            center: Vec2::new(0.0, 0.0),
        }
    }
    
    pub fn get_frame_size(&self) -> (u32, u32) {
        (self.frame_width, self.frame_height)
    }
    
    pub fn center(&mut self) {
        self.center = Vec2::new(self.frame_width as f64 / 2.0, self.frame_height as f64 / 2.0);
    }
    
    pub fn add_animation(&mut self, animation: SpriteAnimation) {
        self.animations.push_back(animation);
    }
    
    pub fn add_named_once(&mut self, name: &String, start: f64, end: f64) {
        let ref frame_interval = self.anim_map[name];
        self.animations.push_back(SpriteAnimation::PlayOnce(start, end, frame_interval.0, frame_interval.1));
    }
    
    pub fn add_named_loop(&mut self, name: &String, start: f64, end: f64, interval: f64) {
        let ref frame_interval = self.anim_map[name];
        self.animations.push_back(SpriteAnimation::Loop(start, end, frame_interval.0, frame_interval.1, interval));
    }
    
    pub fn add_named_stay(&mut self, name: &String, start: f64, end: f64) {
        let ref frame_interval = self.anim_map[name];
        if frame_interval.0 != frame_interval.1 {
            println!("WARNING: stay animation {} doesn't have equal frame start and end values", name);
        }
        self.animations.push_back(SpriteAnimation::Stay(start, end, frame_interval.0));
    }
    
    pub fn draw(&mut self, ctx: &mut Context,
                x: f64, y: f64, rotation: f64, time: f64) -> GameResult<()>
    {
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
                            self.draw_current_frame(ctx, x, y, rotation)?;
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
                            self.draw_current_frame(ctx, x, y, rotation)?;
                        }
                    },
                    SpriteAnimation::Stay(start_time, end_time, frame) => {
                        if time >= start_time {
                            if time <= end_time {
                                self.current_frame = frame;
                            } else {
                                anim_done = true;
                            }
                            self.draw_current_frame(ctx, x, y, rotation)?;
                        }
                    },
                },
            None => {}
        }
        
        if anim_done {
            self.animations.pop_front();
        }

        Ok(())
    }
    
    fn draw_current_frame(&self, ctx: &mut Context,
                          x: f64, y: f64, rotation: f64) -> GameResult<()>
    {
        let frame_w = 1.0 / (self.columns as f32);
        let frame_h = 1.0 / (self.rows as f32);
        let src_x = ((self.current_frame % self.columns) as f32) * frame_w;
        let src_y = ((self.current_frame / self.columns) as f32) * frame_h;

        graphics::draw_ex(
            ctx, &*self.texture,
            DrawParam {
                offset: Point2::new(self.center.x as f32, self.center.y as f32),
                dest: Point2::new(x as f32, y as f32),
                src: Rect::new(src_x, src_y, frame_w, frame_h),
				rotation: rotation as f32,
                ..Default::default()
            });

        Ok(())
    }
    
    pub fn set_frame(&mut self, frame: u32) {
        self.current_frame = frame;
    }
}
