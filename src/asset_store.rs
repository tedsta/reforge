use std::rc::Rc;

use graphics::ImageSize;
use opengl_graphics::Texture;

use assets::{
    SpriteInfo,
    TextureId,
    ENGINE_TEXTURE,
    WEAPON_TEXTURE,
    SHIELD_TEXTURE,
    SOLAR_TEXTURE,
    COMMAND_TEXTURE,
    LASER_TEXTURE,
    EXPLOSION_TEXTURE,
    PROPULSION_TEXTURE,
    GUI_TEXTURE,
};

pub struct AssetStore {
    textures: Vec<Rc<Texture>>,
    sprite_info: Vec<SpriteInfo>
}

impl AssetStore {
    pub fn new() -> AssetStore {
        let textures = vec![
            Rc::new(Texture::from_path(&Path::new("content/textures/modules/engine1.png")).unwrap()),
            Rc::new(Texture::from_path(&Path::new("content/textures/modules/weapon_sprite.png")).unwrap()),
            Rc::new(Texture::from_path(&Path::new("content/textures/modules/shield_sprite.png")).unwrap()),
            Rc::new(Texture::from_path(&Path::new("content/textures/modules/solar_panel_sprite.png")).unwrap()),
            Rc::new(Texture::from_path(&Path::new("content/textures/modules/big_command_sprite.png")).unwrap()),
            Rc::new(Texture::from_path(&Path::new("content/textures/effects/laser1.png")).unwrap()),
            Rc::new(Texture::from_path(&Path::new("content/textures/effects/explosion1.png")).unwrap()),
            Rc::new(Texture::from_path(&Path::new("content/textures/effects/propulsion_sprite.png")).unwrap()),
            Rc::new(Texture::from_path(&Path::new("content/textures/gui/module_button.png")).unwrap()),
        ];
        
        let sprite_info = vec![
            SpriteInfo {
                texture: textures[ENGINE_TEXTURE as uint].clone(),
                columns: 1,
                rows: 1,
            },
            SpriteInfo {
                texture: textures[WEAPON_TEXTURE as uint].clone(),
                columns: 7,
                rows: 1,
            },
            SpriteInfo {
                texture: textures[SHIELD_TEXTURE as uint].clone(),
                columns: 5,
                rows: 2,
            },
            SpriteInfo {
                texture: textures[SOLAR_TEXTURE as uint].clone(),
                columns: 5,
                rows: 3,
            },
            SpriteInfo {
                texture: textures[COMMAND_TEXTURE as uint].clone(),
                columns: 8,
                rows: 1,
            },
            SpriteInfo {
                texture: textures[LASER_TEXTURE as uint].clone(),
                columns: 1,
                rows: 4,
            },
            SpriteInfo {
                texture: textures[EXPLOSION_TEXTURE as uint].clone(),
                columns: 1,
                rows: 10,
            },
            SpriteInfo {
                texture: textures[PROPULSION_TEXTURE as uint].clone(),
                columns: 1,
                rows: 7,
            },
            SpriteInfo {
                texture: textures[GUI_TEXTURE as uint].clone(),
                columns: 1,
                rows: 1,
            },
        ];
    
        AssetStore {
            textures: textures,
            sprite_info: sprite_info,
        }
    }
    
    pub fn get_texture<'a>(&'a self, texture: TextureId) -> &'a Rc<Texture> {
        &self.textures[texture as uint]
    }
    
    pub fn get_texture_size(&self, texture: TextureId) -> (u32, u32) {
        self.textures[texture as uint].get_size()
    }
    
    pub fn get_sprite_info<'a>(&'a self, texture: TextureId) -> &'a SpriteInfo {
        &self.sprite_info[texture as uint]
    }
}