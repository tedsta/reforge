use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;

use graphics::ImageSize;
use opengl_graphics::Texture;
use sdl2_mixer;

pub struct SpriteInfo {
    pub texture: Rc<Texture>,
    pub columns: u8, 
    pub rows: u8,
}

pub struct AssetStore {
    sprite_info: HashMap<String, SpriteInfo>,
    
    sounds: HashMap<String, Rc<RefCell<sdl2_mixer::Chunk>>>,
}

impl AssetStore {
    pub fn new() -> AssetStore {
        let mut sounds = HashMap::new();
        sounds.insert(
            "effects/small_explosion.wav".to_string(),
            Rc::new(RefCell::new(sdl2_mixer::Chunk::from_file(&Path::new("content/audio/effects/small_explosion.wav"))
                .ok().expect("Failed to load sound")))
        );
        sounds.insert(
            "effects/laser.wav".to_string(),
            Rc::new(RefCell::new(sdl2_mixer::Chunk::from_file(&Path::new("content/audio/effects/laser.wav"))
                .ok().expect("Failed to load sound")))
        );
        sounds.insert(
            "effects/beam1.ogg".to_string(),
            Rc::new(RefCell::new(sdl2_mixer::Chunk::from_file(&Path::new("content/audio/effects/beam1.ogg"))
                .ok().expect("Failed to load sound")))
        );
        
        sounds.get_mut("effects/laser.wav").expect("This should exist").borrow_mut().set_volume(32);
        sounds.get_mut("effects/small_explosion.wav").expect("This should exist").borrow_mut().set_volume(32);
    
        let mut asset_store = AssetStore {
            sprite_info: HashMap::new(),
            
            sounds: sounds,
        };
        
        asset_store.load_texture("modules/engine1.png", 1, 1);
        asset_store.load_texture("modules/weapon_sprite.png", 7, 1);
        asset_store.load_texture("modules/shield_sprite.png", 5, 2);
        asset_store.load_texture("modules/solar_panel_sprite.png", 5, 3);
        asset_store.load_texture("modules/repair_sprite.png", 19, 1);
        asset_store.load_texture("modules/big_command_sprite.png", 8, 1);
        asset_store.load_texture("modules/small_beam_sprite.png", 6, 4);
        asset_store.load_texture("effects/laser1.png", 1, 4);
        asset_store.load_texture("effects/explosion1.png", 1, 10);
        asset_store.load_texture("effects/propulsion_sprite.png", 1, 7);
        asset_store.load_texture("effects/fire_sprite.png", 8, 1);
        asset_store.load_texture("effects/smoke_sprite.png", 8, 1);
        asset_store.load_texture("effects/small_beam_part.png", 1, 1);
        asset_store.load_texture("effects/small_beam_end.png", 1, 4);
        asset_store.load_texture("gui/small_target.png", 1, 1);
        asset_store.load_texture("gui/medium_target.png", 1, 1);
        asset_store.load_texture("gui/big_target.png", 1, 1);
        
        asset_store
    }
    
    fn load_texture(&mut self, name: &str, columns: u8, rows: u8) {
        let name = name.to_string();
        let texture_path = "content/textures/".to_string() + &name;
        let texture =
            Rc::new(
                Texture::from_path(&Path::new(texture_path.as_slice()))
                    .ok().expect(format!("Failed to load {}", name).as_slice())
            );
        self.sprite_info.insert(
            name,
            SpriteInfo {
                texture: texture,
                columns: columns,
                rows: rows,
            },
        );
    }
    
    pub fn get_texture<'a>(&'a self, texture: &String) -> &'a Rc<Texture> {
        &self.sprite_info[texture].texture
    }
    
    pub fn get_texture_str<'a>(&'a self, texture: &str) -> &'a Rc<Texture> {
        &self.sprite_info[&texture.to_string()].texture
    }
    
    pub fn get_texture_size(&self, texture: &String) -> (u32, u32) {
        self.sprite_info[texture].texture.get_size()
    }
    
    pub fn get_texture_size_str(&self, texture: &str) -> (u32, u32) {
        self.sprite_info[&texture.to_string()].texture.get_size()
    }
    
    pub fn get_sprite_info<'a>(&'a self, texture: &String) -> &'a SpriteInfo {
        &self.sprite_info[texture]
    }
    
    pub fn get_sprite_info_str<'a>(&'a self, texture: &str) -> &'a SpriteInfo {
        &self.sprite_info[&texture.to_string()]
    }
    
    pub fn get_sound(&self, name: &String) -> &Rc<RefCell<sdl2_mixer::Chunk>> {
        &self.sounds[name]
    }
}