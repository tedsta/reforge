use rsfml::graphics::{Sprite, Texture};

pub struct SpriteSheet {
    // SFML sprite
    sprite: Sprite,
    
    // Sprite sheet info
    columns: u8,
    rows: u8,
    frame_width: u16,
    frame_height: u16,
    
    // Sprite sheet state
    current_frame: u16,
    start_frame: u16,
    stop_frame: u16,
    
    // Time stuff
    interval: f32,
    accumulator: f32,
}

impl SpriteSheet {
    pub fn new(texture: Texture, columns: u8, rows: u8, interval: f32) -> Sprite {
        let size let size = texture.get_size();
        let (texture_width, texture_height) = (size.x as u16, size.y as u16);
        
        Sprite {
            sprite: Sprite::new(texture),
            columns: columns,
            rows: rows,
            frames: frames,
            frame_width: texture_width/(columns as u16),
            frame_height: texture_height/(rows as u16),
            current_frame: 0,
            start_frame: 0,
            stop_frame: (columns as u16)*(rows as u16) - 1,
            interval: interval,
            accumulator: 0.0;
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.accumulator += dt;
        while self.accumulator > interval {
            self.accumulator -= interval;
            self.set_frame(self.current_frame+1)
        }
    }
    
    pub fn set_frame(&mut self, frame: u16) {
        self.current_frame = frame+1;
        self.sprite
    }
}