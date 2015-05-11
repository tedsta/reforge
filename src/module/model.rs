use std::path::Path;

use opengl_graphics::Texture;

use super::{
    ModuleStored,
    
    EngineModule,
};

pub struct Model {
    pub name: String,
    factory: Box<Fn() -> ModuleStored + Sync + Send>,
    
    #[cfg(feature = "client")]
    pub icon: Texture,
    
    pub width: u8,
    pub height: u8,
    pub power: u8,  // Power consumption
    pub min_hp: u8, // Minimum HP for the module to still operate
    pub max_hp: u8, // Maximum HP of module, including armor
}

impl Model {
    pub fn create(&self) -> ModuleStored {
        let ref factory = self.factory;
        factory()
    }
}

pub struct ModelStore {
    models: Vec<Model>,
}

impl ModelStore {
    pub fn new() -> ModelStore {
        let models =
            vec![
                Model {
                    name: "Engine Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(EngineModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/engine1.png")).unwrap(),
                    width: 2,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
            ];
    
        ModelStore {
            models: models,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, RustcEncodable, RustcDecodable)]
pub struct ModelIndex(pub u16);

impl ModelIndex {
    pub fn get<'a>(self, model_store: &'a ModelStore) -> &'a Model {
        &model_store.models[self.0 as usize]
    }
}