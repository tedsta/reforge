pub trait Renderer {
    fn draw_texture(&mut self, texture: TextureId);
    fn draw_texture_on_target(&mut self, target: RenderTargetId, texture: TextureId);
}

pub type RenderTargetId = uint;

pub enum TextureId {
    Engine = 0,
    TextureCount,
}