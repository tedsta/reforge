use std::collections::HashMap;
use std::rc::Rc;
use std::string::String;

use graphics::ImageSize;
use opengl_graphics::Texture;
use sdl2_mixer;

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
    texture_ids: HashMap<String, TextureId>,
    textures: Vec<Rc<Texture>>,
    sprite_info: Vec<SpriteInfo>,
    
    sounds: HashMap<String, Rc<sdl2_mixer::Chunk>>,
}

impl AssetStore {
    pub fn new() -> AssetStore {
        let mut texture_ids = HashMap::new();
        texture_ids.insert(String::from_str("WEAPON"), WEAPON_TEXTURE);
    
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
        
        let mut sounds = HashMap::new();
        sounds.insert(
            "content/audio/effects/small_explosion.wav".to_string(),
            Rc::new(sdl2_mixer::Chunk::from_file(&Path::new("content/audio/effects/small_explosion.wav"))
                .ok().expect("Failed to load sound"))
        );
        sounds.insert(
            "content/audio/effects/laser.wav".to_string(),
            Rc::new(sdl2_mixer::Chunk::from_file(&Path::new("content/audio/effects/laser.wav"))
                .ok().expect("Failed to load sound"))
        );
    
        AssetStore {
            texture_ids: texture_ids,
            textures: textures,
            sprite_info: sprite_info,
            
            sounds: sounds,
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
    
    pub fn get_sound(&self, name: &String) -> &Rc<sdl2_mixer::Chunk> {
        &self.sounds[*name]
    }
}