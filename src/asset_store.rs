use rsfml::graphics::Texture;

use assets::TextureId;

pub struct AssetStore {
    textures: Vec<Texture>,
}

impl AssetStore {
    pub fn new() -> AssetStore {
        let textures = vec![
            Texture::new_from_file("content/textures/modules/engine1.png").expect("Failed to load texture"),
            Texture::new_from_file("content/textures/modules/laser1.png").expect("Failed to load texture"),
        ];
    
        AssetStore {
            textures: textures,
        }
    }
    
    pub fn get_texture<'a>(&'a self, texture: TextureId) -> &'a Texture {
        &self.textures[texture as uint]
    }
}