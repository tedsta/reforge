use vec::{Vec2f};

pub trait Renderer {
    fn get_texture_info<'a>(&'a self, texture: TextureId) -> &'a TextureInfo;

    fn draw_texture(&self, texture: TextureId, x: f32, y: f32);
    
    // This is available to draw things using vectors
    fn draw_texture_vec(&self, texture: TextureId, pos: &Vec2f) {
        self.draw_texture(texture, pos.x, pos.y);
    }
    
    fn draw_texture_target(&self, target: RenderTargetId, texture: TextureId, x: f32, y: f32);
}

#[deriving(Encodable, Decodable, Default)]
pub struct RenderTarget {
    pub id: RenderTargetId,
    pub texture: TextureId,
}

impl RenderTarget {
    pub fn draw_texture(&self, renderer: &Renderer, texture: TextureId, x: f32, y: f32) {
        renderer.draw_texture_target(self.id, texture, x, y);
    }
    
    pub fn draw_texture_vec(&self, renderer: &Renderer, texture: TextureId, pos: &Vec2f) {
        self.draw_texture(renderer, texture, pos.x, pos.y);
    }
}

pub type RenderTargetId = u16;

pub type TextureId = u16;

pub struct TextureInfo {
    pub width: u16,
    pub height: u16,
}

pub static ENGINE_TEXTURE: u16 = 0;
pub static LASER_TEXTURE: u16 = 1;

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