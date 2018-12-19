use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use std::collections::HashMap;
use std::rc::Rc;
use std::path::Path;

use ggez::{audio, Context, GameResult};
use ggez::graphics::Image;

use config;

pub struct SpriteInfo {
    pub texture: Rc<Image>,
    pub columns: u8, 
    pub rows: u8,
    pub animations: HashMap<String, (u32, u32)>,
}

pub struct AssetStore {
    sprite_info: HashMap<String, SpriteInfo>,
    
    sounds: HashMap<String, Rc<audio::Source>>,
}

impl AssetStore {
    pub fn new(ctx: &mut Context) -> GameResult<AssetStore> {
        let mut laser_snd = audio::Source::new(ctx, "/audio/effects/laser.wav")?;
        laser_snd.set_volume(0.5);
        let mut small_explosion_snd =
            audio::Source::new(ctx, "/audio/effects/small_explosion.wav")?;
        small_explosion_snd.set_volume(0.5);

        let mut sounds = HashMap::new();
        sounds.insert(
            "effects/small_explosion.wav".to_string(),
            Rc::new(small_explosion_snd));
        sounds.insert(
            "effects/laser.wav".to_string(),
            Rc::new(laser_snd));
        sounds.insert(
            "effects/beam1.ogg".to_string(),
            Rc::new(audio::Source::new(ctx, "/audio/effects/beam1.ogg")?));
        sounds.insert(
            "effects/ship_explosion1.ogg".to_string(),
            Rc::new(audio::Source::new(ctx, "/audio/effects/ship_explosion1.ogg")?));
        
        let mut asset_store = AssetStore {
            sprite_info: HashMap::new(),
            
            sounds: sounds,
        };

        println!("Loading sprites...");
        
        // Read module models from text files
        let paths = fs::read_dir(&Path::new("resources/data/sprites")).unwrap();
        for path in paths.map(|p| p.unwrap().path()) {
            if path.is_file() {
                asset_store.load_sprite(
                    ctx, &config::read_properties(BufReader::new(File::open(&path).unwrap())));
            }
        }
        
        println!("Done loading stuff");
        
        /*asset_store.load_texture("modules/engine1.png", 1, 1);
        asset_store.load_texture("modules/weapon_sprite.png", 7, 1);
        asset_store.load_texture("modules/shield_sprite.png", 5, 2);
        asset_store.load_texture("modules/solar_panel_sprite.png", 5, 3);
        asset_store.load_texture("modules/repair_sprite.png", 19, 1);
        asset_store.load_texture("modules/big_command_sprite.png", 8, 1);
        asset_store.load_texture("modules/cabin.png", 1, 1);
        asset_store.load_texture("modules/small_beam_sprite.png", 6, 4);
        asset_store.load_texture("modules/pewpewbase.png", 1, 1);
        asset_store.load_texture("modules/pewpewfire.png", 15, 1);
        asset_store.load_texture("effects/laser1.png", 1, 4);
        asset_store.load_texture("effects/laser2.png", 4, 1);
        asset_store.load_texture("effects/explosion1.png", 1, 10);
        asset_store.load_texture("effects/propulsion_sprite.png", 1, 7);
        asset_store.load_texture("effects/fire_sprite.png", 8, 1);
        asset_store.load_texture("effects/smoke_sprite.png", 8, 1);
        asset_store.load_texture("effects/small_beam_part.png", 1, 1);
        asset_store.load_texture("effects/small_beam_end.png", 1, 4);
        asset_store.load_texture("effects/1_module_shield.png", 1, 1);
        asset_store.load_texture("effects/ship_explosion1.png", 9, 1);
        asset_store.load_texture("gui/small_target.png", 1, 1);
        asset_store.load_texture("gui/medium_target.png", 1, 1);
        asset_store.load_texture("gui/big_target.png", 1, 1);*/
        
        Ok(asset_store)
    }
    
    fn load_sprite(&mut self, ctx: &mut Context, prop: &HashMap<String, String>) -> GameResult<()> {
        println!("loading {:?}", prop);
    
        let name = prop["name"].clone();
        
        let texture_path = "/textures/".to_string() + &prop["texture"];
        let rows = prop["rows"].parse().unwrap();
        let columns = prop["columns"].parse().unwrap();
        let mut texture = Image::new(ctx, texture_path.as_str())?;
        // Default to wrapping textures
        // TODO can we wrap a single tile in a sprite sheet?
        texture.set_wrap(
            gfx_core::texture::WrapMode::Tile, gfx_core::texture::WrapMode::Tile);
        let mut anim_map = HashMap::new();
        if prop.contains_key("animations") {
            let animations: Vec<String> = prop["animations"].split("\n")
                                                            .map(|s| s.trim_left().trim_right().to_string())
                                                            .collect();
            for animation in animations {
                let parts: Vec<String> = animation.split(":")
                                                  .map(|s| s.trim_left().trim_right().to_string())
                                                  .collect();
                if parts.len() == 3 {
                    anim_map.insert(parts[0].clone(), (parts[1].parse().unwrap(), parts[2].parse().unwrap()));
                }
            }
        }
        println!("animations: {:?}", anim_map);
        self.sprite_info.insert(
            name,
            SpriteInfo {
                texture: Rc::new(texture),
                columns: columns,
                rows: rows,
                animations: anim_map,
            });
        
        Ok(())
    }
    
    pub fn get_texture<'a>(&'a self, texture: &String) -> &'a Rc<Image> {
        &self.sprite_info[texture].texture
    }
    
    pub fn get_texture_str<'a>(&'a self, texture: &str) -> &'a Rc<Image> {
        &self.sprite_info[&texture.to_string()].texture
    }
    
    pub fn get_texture_size(&self, texture: &String) -> (u32, u32) {
        let ref texture = self.sprite_info[texture].texture;
        (texture.width(), texture.height())
    }
    
    pub fn get_texture_size_str(&self, texture: &str) -> (u32, u32) {
        let ref texture = self.sprite_info[&texture.to_string()].texture;
        (texture.width(), texture.height())
    }
    
    pub fn get_sprite_info<'a>(&'a self, texture: &String) -> &'a SpriteInfo {
        &self.sprite_info[texture]
    }
    
    pub fn get_sprite_info_str<'a>(&'a self, texture: &str) -> &'a SpriteInfo {
        &self.sprite_info[&texture.to_string()]
    }
    
    pub fn get_sound(&self, name: &String) -> &Rc<audio::Source> {
        &self.sounds[name]
    }
}
