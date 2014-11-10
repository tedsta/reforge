use std::rc::Rc;

use graphics::ImageSize;
use opengl_graphics::Texture;

use assets::{SpriteInfo, TextureId, ENGINE_TEXTURE, WEAPON_TEXTURE, LASER_TEXTURE, EXPLOSION_TEXTURE, GUI_TEXTURE};

pub struct AssetStore {
    textures: Vec<Rc<Texture>>,
    sprite_info: Vec<SpriteInfo>
}

impl AssetStore {
    pub fn new() -> AssetStore {
        let textures = vec![
            Rc::new(Texture::from_path(&Path::new("content/textures/modules/engine1.png")).expect("Failed to load texture")),
            Rc::new(Texture::from_path(&Path::new("content/textures/modules/weapon_sprite.png")).expect("Failed to load texture")),
            Rc::new(Texture::from_path(&Path::new("content/textures/effects/laser1.png")).expect("Failed to load texture")),
            Rc::new(Texture::from_path(&Path::new("content/textures/effects/explosion1.png")).expect("Failed to load texture")),
            Rc::new(Texture::from_path(&Path::new("content/textures/gui/module_button.png")).expect("Failed to load texture")),
        ];
        
        let sprite_info = vec![
            SpriteInfo {
                texture: ENGINE_TEXTURE,
                texture_width: textures[ENGINE_TEXTURE as uint].get_width() as u16,
                texture_height: textures[ENGINE_TEXTURE as uint].get_height() as u16,
                columns: 1,
                rows: 1,
            },
            SpriteInfo {
                texture: WEAPON_TEXTURE,
                texture_width: textures[WEAPON_TEXTURE as uint].get_width() as u16,
                texture_height: textures[WEAPON_TEXTURE as uint].get_height() as u16,
                columns: 6,
                rows: 1,
            },
            SpriteInfo {
                texture: LASER_TEXTURE,
                texture_width: textures[LASER_TEXTURE as uint].get_width() as u16,
                texture_height: textures[LASER_TEXTURE as uint].get_height() as u16,
                columns: 1,
                rows: 4,
            },
            SpriteInfo {
                texture: EXPLOSION_TEXTURE,
                texture_width: textures[EXPLOSION_TEXTURE as uint].get_width() as u16,
                texture_height: textures[EXPLOSION_TEXTURE as uint].get_height() as u16,
                columns: 1,
                rows: 10,
            },
            SpriteInfo {
                texture: GUI_TEXTURE,
                texture_width: textures[GUI_TEXTURE as uint].get_width() as u16,
                texture_height: textures[GUI_TEXTURE as uint].get_height() as u16,
                columns: 1,
                rows: 1,
            },
        ];
    
        AssetStore {
            textures: textures,
            sprite_info: sprite_info,
        }
    }
    
    pub fn get_texture<'a>(&'a self, texture: TextureId) -> &'a Texture {
        &self.textures[texture as uint]
    }
    
    pub fn get_texture_size(&self, texture: TextureId) -> (u32, u32) {
        self.textures[texture as uint].get_size()
    }
    
    pub fn get_sprite_info<'a>(&'a self, texture: TextureId) -> &'a SpriteInfo {
        &self.sprite_info[texture as uint]
    }
}