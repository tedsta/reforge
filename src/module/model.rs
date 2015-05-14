use std::path::Path;

#[cfg(feature = "client")]
use opengl_graphics::Texture;

use super::{
    ModuleStored,
    
    EngineModule,
    ProjectileWeaponModule,
    ShieldModule,
    SolarModule,
    CommandModule,
    BeamWeaponModule,
    RepairModule,
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
    #[cfg(feature = "client")]
    pub fn new() -> ModelStore {
        let models =
            vec![
                Model {
                    name: "Engine Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(EngineModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/engine.png")).unwrap(),
                    width: 2,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Command Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(CommandModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/command.png")).unwrap(),
                    width: 1,
                    height: 2,
                    power: 2,
                    min_hp: 2,
                    max_hp: 4,
                },
                Model {
                    name: "Solar Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(SolarModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/solar.png")).unwrap(),
                    width: 1,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Shield Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(ShieldModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/shield.png")).unwrap(),
                    width: 1,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Blaster Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(ProjectileWeaponModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/blaster.png")).unwrap(),
                    width: 1,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Beam Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(BeamWeaponModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/beam.png")).unwrap(),
                    width: 1,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Repair Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(RepairModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/repair.png")).unwrap(),
                    width: 1,
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
    
    #[cfg(feature = "server")]
    pub fn new() -> ModelStore {
        let models =
            vec![
                Model {
                    name: "Engine Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(EngineModule::new())),
                    width: 2,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Command Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(CommandModule::new())),
                    width: 1,
                    height: 2,
                    power: 2,
                    min_hp: 2,
                    max_hp: 4,
                },
                Model {
                    name: "Solar Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(SolarModule::new())),
                    width: 1,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Shield Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(ShieldModule::new())),
                    width: 1,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Blaster Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(ProjectileWeaponModule::new())),
                    width: 1,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Beam Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(BeamWeaponModule::new())),
                    width: 1,
                    height: 1,
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Repair Mk1".to_string(),
                    factory: Box::new(move || ModuleStored::from_module(RepairModule::new())),
                    width: 1,
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