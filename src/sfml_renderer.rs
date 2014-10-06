use std::string::String;

use rsfml::graphics::{Color, RenderStates, RenderTarget, RenderTexture, RenderWindow, Sprite, Texture, Vertex, Quads};
use rsfml::system::{Vector2f};

use render;
use render::{Renderer, RenderTargetId, TextureId, TextureInfo};

pub struct SfmlRenderer {
    pub window: RenderWindow,
    render_textures: Vec<RenderTexture>,
    
    textures: Vec<Texture>,
    texture_info: Vec<TextureInfo>,
}

impl SfmlRenderer {
    pub fn new(window: RenderWindow) -> SfmlRenderer {
        let textures = vec![
            Texture::new_from_file("content/textures/modules/engine1.png").expect("Failed to load texture"),
            Texture::new_from_file("content/textures/modules/laser1.png").expect("Failed to load texture"),
        ];
        
        let mut texture_info = Vec::with_capacity(2);
        for texture in textures.iter() {
            texture_info.push(TextureInfo{width: texture.get_size().x as u16, height: texture.get_size().y as u16});
        }
    
        SfmlRenderer {
            window: window,
            render_textures: vec!(),

            textures: textures,
            texture_info: texture_info,
        }
    }
    
    pub fn clear_render_targets(&mut self) {
        for r in self.render_textures.iter_mut() {
            r.clear(&Color::new_RGBA(150, 70, 30, 120));
        }
    }
    
    pub fn display_render_targets(&mut self) {
        for r in self.render_textures.iter_mut() {
            r.display();
        }
    }
    
    pub fn create_render_target(&mut self, width: u32, height: u32) -> render::RenderTarget {
        let rt = RenderTexture::new(width as uint, height as uint, false).expect("Failed to create render texture");
        let texture_id = self.add_texture(rt.get_texture().expect("Failed to get RenderTexture's texture"));
        self.render_textures.push(rt);
        
        render::RenderTarget{id: (self.render_textures.len() - 1) as u16, texture: texture_id}
    }
    
    fn add_texture(&mut self, texture: Texture) -> TextureId {
        self.texture_info.push(TextureInfo{width: texture.get_size().x as u16, height: texture.get_size().y as u16});
        self.textures.push(texture);
        (self.textures.len() - 1) as u16
    }
}

impl Renderer for SfmlRenderer {
    fn get_texture_info<'a>(&'a self, texture: TextureId) -> &'a TextureInfo {
        &self.texture_info[texture as uint]
    }

    fn draw_texture(&self, texture_id: TextureId, x: f32, y: f32) {
        let texture = &self.textures[texture_id as uint];
        
        let size = texture.get_size();
        let (width, height) = (size.x as f32, size.y as f32);

        let vertices = [
            Vertex::new(&Vector2f{x: x, y: y}, &Color::white(), &Vector2f{x: 0f32, y: 0f32}),
            Vertex::new(&Vector2f{x: x, y: y + height}, &Color::white(), &Vector2f{x: 0f32, y: height}),
            Vertex::new(&Vector2f{x: x + width, y: y + height}, &Color::white(), &Vector2f{x: width, y: height}),
            Vertex::new(&Vector2f{x: x + width, y: y}, &Color::white(), &Vector2f{x: width, y: 0f32})
        ];
        
        let mut rs = RenderStates::default();
        rs.texture = Some(texture);
        
        (&self.window as &RenderTarget).draw_primitives_rs(&vertices, Quads, &mut rs);
    }
    
    fn draw_texture_target(&self, target: RenderTargetId, texture: TextureId, x: f32, y: f32) {
        let render_texture = &self.render_textures[target as uint];
        
        let texture = &self.textures[texture as uint];
        
        let size = texture.get_size();
        let (width, height) = (size.x as f32, size.y as f32);

        let vertices = [
            Vertex::new(&Vector2f{x: x, y: y}, &Color::white(), &Vector2f{x: 0f32, y: 0f32}),
            Vertex::new(&Vector2f{x: x, y: y + height}, &Color::white(), &Vector2f{x: 0f32, y: height}),
            Vertex::new(&Vector2f{x: x + width, y: y + height}, &Color::white(), &Vector2f{x: width, y: height}),
            Vertex::new(&Vector2f{x: x + width, y: y}, &Color::white(), &Vector2f{x: width, y: 0f32})
        ];
        
        let mut rs = RenderStates::default();
        rs.texture = Some(texture);
        
        (render_texture as &RenderTarget).draw_primitives_rs(&vertices, Quads, &mut rs);
    }
}