pub type RenderTargetId = uint;
pub type TextureId = uint;

pub trait Renderer {
    fn draw_texture(&mut self, target: RenderTargetId, texture: TextureId);
}