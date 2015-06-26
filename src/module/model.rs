use std::collections::HashMap;
use std::fs;
use std::fs::{File, PathExt};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::str;

#[cfg(feature = "client")]
use opengl_graphics::Texture;

use super::{
    ModuleShape,
    ModuleStored,
    
    EngineModule,
    ProjectileWeaponModule,
    ShieldModule,
    SolarModule,
    CommandModule,
    CabinModule,
    BeamWeaponModule,
    RepairModule,
};
use config;

pub struct Model {
    pub name: String,
    factory: Box<Fn(&Model) -> ModuleStored + Sync + Send>,
    
    #[cfg(feature = "client")]
    pub icon: Texture,
    
    pub shape: ModuleShape,
    pub power: u8,  // Power consumption
    pub min_hp: u8, // Minimum HP for the module to still operate
    pub max_hp: u8, // Maximum HP of module, including armor
}

impl Model {
    pub fn create(&self) -> ModuleStored {
        let ref factory = self.factory;
        factory(self)
    }
}

pub struct ModelStore {
    models: Vec<Model>,
}

impl ModelStore {
    #[cfg(feature = "client")]
    pub fn new() -> ModelStore {
        let mut models =
            vec![
                Model {
                    name: "Engine Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(EngineModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/engine.png")).unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#', b'#'],
                                                 vec![b'.', b'.']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Command Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(CommandModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/command.png")).unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#', b'.'],
                                                 vec![b'#', b'.']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 4,
                },
                Model {
                    name: "Solar Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(SolarModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/solar.png")).unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Shield Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(ShieldModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/shield.png")).unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Blaster Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(ProjectileWeaponModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/pewpew.png")).unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Beam Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(BeamWeaponModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/beam.png")).unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Repair Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(RepairModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/repair.png")).unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Cabin Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(CabinModule::new())),
                    icon: Texture::from_path(&Path::new("content/textures/modules/icons/cabin.png")).unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#', b'.', b'.'],
                                                 vec![b'#', b'#', b'.'],
                                                 vec![b'#', b'.', b'.']]),
                    power: 0,
                    min_hp: 3,
                    max_hp: 8,
                },
            ];
        
        // Read module models from text files
        let paths = fs::read_dir(&Path::new("content/data/modules")).unwrap();
        for path in paths.map(|p| p.unwrap().path()) {
            if path.is_file() {
                models.push(model_from_properties(&config::read_properties(BufReader::new(File::open(&path).unwrap()))));
            }
        }
    
        ModelStore {
            models: models,
        }
    }
    
    #[cfg(feature = "server")]
    pub fn new() -> ModelStore {
        let mut models =
            vec![
                Model {
                    name: "Engine Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(EngineModule::new())),
                    shape: ModuleShape::new(vec![vec![b'#', b'#'],
                                                 vec![b'.', b'.']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Command Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(CommandModule::new())),
                    shape: ModuleShape::new(vec![vec![b'#', b'.'],
                                                 vec![b'#', b'.']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 4,
                },
                Model {
                    name: "Solar Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(SolarModule::new())),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Shield Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(ShieldModule::new())),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Blaster Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(ProjectileWeaponModule::new())),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Beam Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(BeamWeaponModule::new())),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Repair Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(RepairModule::new())),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    name: "Cabin Mk1".to_string(),
                    factory: Box::new(move |_| ModuleStored::from_module(CabinModule::new())),
                    shape: ModuleShape::new(vec![vec![b'#', b'.', b'.'],
                                                 vec![b'#', b'#', b'.'],
                                                 vec![b'#', b'.', b'.']]),
                    power: 0,
                    min_hp: 3,
                    max_hp: 8,
                },
            ];
        
        // Read module models from text files
        let paths = fs::read_dir(&Path::new("content/data/modules")).unwrap();
        for path in paths.map(|p| p.unwrap().path()) {
            if path.is_file() {
                models.push(model_from_properties(&config::read_properties(BufReader::new(File::open(&path).unwrap()))));
            }
        }
    
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

#[cfg(feature = "client")]
fn model_from_properties(prop: &HashMap<String, String>) -> Model {
    let factory = factory_from_properties(prop);
    
    let shape: Vec<Vec<u8>> =
        prop["shape"].lines()
                     .map(|l| l.trim_left().trim_right().bytes().collect())
                     .collect();
    
    Model {
        name: prop["name"].clone(),
        factory: factory,
        icon: Texture::from_path(&Path::new(prop["icon"].as_str())).unwrap(),
        shape: ModuleShape::new(shape),
        power: prop["power"].parse().unwrap(),
        min_hp: prop["min_hp"].parse().unwrap(),
        max_hp: prop["max_hp"].parse().unwrap(),
    }
}

#[cfg(feature = "server")]
fn model_from_properties(prop: &HashMap<String, String>) -> Model {
    let factory = factory_from_properties(prop);
    
    let shape: Vec<Vec<u8>> =
        prop["shape"].lines()
                     .map(|l| l.trim_right()) // Trim newline
                     .filter(|l| l.len() > 0)
                     .map(|l| l.bytes().collect())
                     .collect();
    
    Model {
        name: prop["name"].clone(),
        factory: factory,
        shape: ModuleShape::new(shape),
        power: prop["power"].parse().unwrap(),
        min_hp: prop["min_hp"].parse().unwrap(),
        max_hp: prop["max_hp"].parse().unwrap(),
    }
}

fn factory_from_properties(prop: &HashMap<String, String>)
    -> Box<Fn(&Model) -> ModuleStored + Sync + Send> 
{
    let module_class = prop["class"].as_str();
    let prop_cloned = prop.clone();
    match module_class {
        "ProjectileWeapon" => {
            Box::new(move |model| {
                ModuleStored::from_module(ProjectileWeaponModule::from_properties(model, &prop_cloned))
            })
        },
        "BeamWeapon" => {
            Box::new(move |_| {
                ModuleStored::from_module(BeamWeaponModule::new())
            })
        },
        "Command" => {
            Box::new(move |_| {
                ModuleStored::from_module(CommandModule::new())
            })
        },
        "Cabin" => {
            Box::new(move |_| {
                ModuleStored::from_module(CabinModule::new())
            })
        },
        "Shield" => {
            Box::new(move |_| {
                ModuleStored::from_module(ShieldModule::new())
            })
        },
        "Solar" => {
            Box::new(move |_| {
                ModuleStored::from_module(SolarModule::new())
            })
        },
        "Repair" => {
            Box::new(move |_| {
                ModuleStored::from_module(RepairModule::new())
            })
        },
        "Engine" => {
            Box::new(move |_| {
                ModuleStored::from_module(EngineModule::new())
            })
        },
        _ => {
            panic!("Unknown module class: {}", module_class);
        },
    }
}