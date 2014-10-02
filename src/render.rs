use vec::{Vec2f};

pub trait Renderer {
    fn draw_texture(&mut self, texture: TextureId, x: f32, y: f32);
    
    // This is available to draw things using vectors
    fn draw_texture_vec(&mut self, texture: TextureId, pos: &Vec2f) {
        self.draw_texture(texture, pos.x, pos.y);
    }
    
    fn draw_texture_on_target(&mut self, target: RenderTargetId, texture: TextureId);
}

pub type RenderTargetId = u32;

#[deriving(FromPrimitive)]
pub enum TextureId {
    Engine = 0,
    TextureCount,
}

pub struct Sprite {
    // Texture
    texture: TextureId,
    
    // Sprite sheet info
    frames_per_row: uint,
    frame_rows: uint,
    frames: uint,
    frame_width: uint,
    frame_height: uint,
    
    // Sprite sheet state
    current_frame: uint,
    start_frame: uint,
    stop_frame: uint,
}

impl Sprite {
    pub fn new(texture: TextureId, frames_per_row: uint, frame_rows: uint, texture_width: uint, texture_height: uint) -> Sprite {
        Sprite {
            texture: texture,
            frames_per_row: frames_per_row,
            frame_rows: frame_rows,
            frames: frames_per_row*frame_rows,
            frame_width: texture_width/frames_per_row,
            frame_height: texture_height/frame_rows,
            current_frame: 0,
            start_frame: 0,
            stop_frame: frames_per_row*frame_rows - 1,
        }
    }
}