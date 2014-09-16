use rsfml::graphics::{RenderTexture, RenderWindow, Sprite};

use render::{Renderer, RenderTargetId, TextureId};

pub struct SfmlRenderer<'a> {
    pub window: RenderWindow,
    render_textures: Vec<RenderTexture>,
    render_sprites: Vec<Sprite<'a>>,
    
    textures: Vec<Sprite<'a>>,
}

impl<'a> SfmlRenderer<'a> {
    pub fn new(window: RenderWindow) -> SfmlRenderer<'a> {
        SfmlRenderer {
            window: window,
            render_textures: vec!(),
            render_sprites: vec!(),
            textures: vec!(),
        }
    }
    
    pub fn render(&mut self) {
        for r in self.render_textures.mut_iter() {
            r.display();
        }
        self.window.display();
    }
}

impl<'a> Renderer for SfmlRenderer<'a> {
    fn draw_texture(&mut self, target: RenderTargetId, texture: TextureId) {
        let sprite = self.textures.get_mut(texture);
        self.render_textures.get_mut(target).draw(sprite);
    }
}