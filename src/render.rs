pub trait Renderer {
    fn draw_texture(&mut self, texture: TextureId, x: f32, y: f32);
    fn draw_texture_on_target(&mut self, target: RenderTargetId, texture: TextureId);
}

pub type RenderTargetId = uint;

#[deriving(FromPrimitive)]
pub enum TextureId {
    Engine = 0,
    TextureCount,
}