use graphics::Context;
use opengl_graphics::Gl;

use sprite_sheet::SpriteSheet;

#[derive(Copy, Clone)]
pub struct SpriteIndex(usize);

impl SpriteIndex {
    pub fn get<'a>(self, sprite_mgr: &'a SpriteManager) -> &'a SpriteSheet {
        sprite_mgr.sprites[self.0].as_ref().expect("Failed to access non-existant sprite")
    }
    
    pub fn get_mut<'a>(self, sprite_mgr: &'a mut SpriteManager) -> &'a mut SpriteSheet {
        sprite_mgr.sprites[self.0].as_mut().expect("Failed to access non-existant sprite")
    }
}

pub struct SpriteManager {
    sprites: Vec<Option<SpriteSheet>>,
    free_indices: Vec<usize>,
}

impl SpriteManager {
    pub fn new() -> SpriteManager {
        SpriteManager {
            sprites: vec!(),
            free_indices: vec!(),
        }
    }
    
    pub fn add(&mut self, sprite: SpriteSheet) -> SpriteIndex {
        if let Some(index) = self.free_indices.pop() {
            if self.sprites[index].is_some() {
                // This shouldn't be possible
                panic!("Free sprite index points to existing sprite.");
            }
            
            self.sprites[index] = Some(sprite);
            SpriteIndex(index)
        } else {
            self.sprites.push(Some(sprite));
            SpriteIndex(self.sprites.len() - 1)
        }
    }
    
    pub fn remove(&mut self, index: SpriteIndex) {
        if self.sprites[index.0].is_none() {
            panic!("Tried to remove non-existant sprite");
        }
        
        self.sprites[index.0] = None;
        self.free_indices.push(index.0);
    }
}