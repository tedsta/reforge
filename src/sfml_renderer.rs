use std::string::String;

use rsfml::graphics::{Color, RenderStates, RenderTarget, RenderTexture, RenderWindow, Sprite, Texture, Vertex, Quads};
use rsfml::system::{Vector2f};

use render::{Renderer, RenderTargetId, TextureId, TextureCount};

pub struct SfmlRenderer {
    pub window: RenderWindow,
    render_textures: Vec<RenderTexture>,
    
    textures: Vec<Texture>,
}

impl SfmlRenderer {
    pub fn new(window: RenderWindow) -> SfmlRenderer {
        let textures = vec![
            Texture::new_from_file("content/textures/modules/test1.png").expect("Failed to load texture"),
        ];
    
        SfmlRenderer {
            window: window,
            render_textures: vec!(),

            textures: textures,
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
}

impl Renderer for SfmlRenderer {
    fn draw_texture(&mut self, texture_id: TextureId, x: f32, y: f32) {
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
    
    fn draw_texture_on_target(&mut self, target: RenderTargetId, texture: TextureId) {
        //let sprite = self.textures.get_mut(texture);
        //self.render_textures.get_mut(target).draw(sprite);
    }
}