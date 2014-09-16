use std::string::String;

use rsfml::graphics::{RenderTexture, RenderWindow, Sprite, Texture};
use rsfml::system::{Vector2f};

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
    
    pub fn display(&mut self) {
        for r in self.render_textures.mut_iter() {
            r.display();
        }
        self.window.display();
    }
    
    pub fn create_render_target(&mut self, x: uint, y: uint, width: uint, height: uint) -> RenderTargetId {
        /*let rt = RenderTexture::new(width, height, false).unwrap();
        let mut r_sprite = Sprite::new_with_texture(&rt.get_texture().unwrap()).unwrap();
        r_sprite.set_position(&Vector2f::new(x as f32, y as f32));
        self.render_textures.push(rt);
        self.render_sprites.push(r_sprite);
        
        self.render_textures.len() - 1*/
        
        0
    }
    
    pub fn load_texture(&mut self, path: String) -> TextureId {
        let texture = Texture::new_from_file(path.as_slice()).unwrap();
        self.textures.push(texture);
        
        self.textures.len() - 1
    }
}

impl<'a> Renderer for SfmlRenderer<'a> {
    fn draw_texture(&mut self, texture: TextureId) {
        let sprite = self.textures.get_mut(texture);
        self.window.draw(sprite);
    }
    
    fn draw_texture_on_target(&mut self, target: RenderTargetId, texture: TextureId) {
        let sprite = self.textures.get_mut(texture);
        self.render_textures.get_mut(target).draw(sprite);
    }
}