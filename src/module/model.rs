use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::str;

#[cfg(feature = "client")]
use ggez::{Context, GameResult, graphics::Image};

use super::{
    ModuleShape,
    Module,
    
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
    pub index: ModelIndex,

    pub name: String,
    factory: Box<Fn(&Model) -> Module + Sync + Send>,
    
    #[cfg(feature = "client")]
    pub icon: Image,
    
    pub shape: ModuleShape,
    pub power: u8,  // Power consumption
    pub min_hp: u8, // Minimum HP for the module to still operate
    pub max_hp: u8, // Maximum HP of module, including armor
}

impl Model {
    pub fn create(&self) -> Module {
        let ref factory = self.factory;
        factory(self)
    }
}

pub struct ModelStore {
    models: Vec<Model>,
}

impl ModelStore {
    #[cfg(feature = "client")]
    pub fn new(ctx: &mut Context) -> ModelStore {
        let mut models =
            vec![
                Model {
                    index: ModelIndex(0),
                    name: "Engine Mk1".to_string(),
                    factory: Box::new(move |_| EngineModule::new(ModelIndex(0))),
                    icon: Image::new(ctx, "/textures/modules/icons/engine.png").unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#', b'#'],
                                                 vec![b'.', b'.']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(1),
                    name: "Command Mk1".to_string(),
                    factory: Box::new(move |_| CommandModule::new(ModelIndex(1))),
                    icon: Image::new(ctx, "/textures/modules/icons/command.png").unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#', b'.'],
                                                 vec![b'#', b'.']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 4,
                },
                Model {
                    index: ModelIndex(2),
                    name: "Solar Mk1".to_string(),
                    factory: Box::new(move |_| SolarModule::new(ModelIndex(2))),
                    icon: Image::new(ctx, "/textures/modules/icons/solar.png").unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(3),
                    name: "Shield Mk1".to_string(),
                    factory: Box::new(move |_| ShieldModule::new(ModelIndex(3))),
                    icon: Image::new(ctx, "/textures/modules/icons/shield.png").unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(4),
                    name: "Blaster Mk1".to_string(),
                    factory: Box::new(move |_| ProjectileWeaponModule::new(ModelIndex(4))),
                    icon: Image::new(ctx, "/textures/modules/icons/pewpew.png").unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(5),
                    name: "Beam Mk1".to_string(),
                    factory: Box::new(move |_| BeamWeaponModule::new(ModelIndex(5))),
                    icon: Image::new(ctx, "/textures/modules/icons/beam.png").unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(6),
                    name: "Repair Mk1".to_string(),
                    factory: Box::new(move |_| RepairModule::new(ModelIndex(6))),
                    icon: Image::new(ctx, "/textures/modules/icons/repair.png").unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(7),
                    name: "Cabin Mk1".to_string(),
                    factory: Box::new(move |_| CabinModule::new(ModelIndex(7))),
                    icon: Image::new(ctx, "/textures/modules/icons/cabin.png").unwrap(),
                    shape: ModuleShape::new(vec![vec![b'#', b'.', b'.'],
                                                 vec![b'#', b'#', b'.'],
                                                 vec![b'#', b'.', b'.']]),
                    power: 0,
                    min_hp: 3,
                    max_hp: 8,
                },
            ];
        
        let mut model_store = ModelStore { models: models };
        
        println!("about to load models client");
        
        // Read module models from text files
        let paths = fs::read_dir("resources/data/modules").unwrap();
        for path in paths.map(|p| p.unwrap().path()) {
            if path.is_file() {
                println!("pre adding model");
                model_store.add_model_from_properties(ctx, &config::read_properties(BufReader::new(File::open(&path).unwrap())));
            }
        }
        
        println!("loaded models client");
    
        model_store
    }
    
    #[cfg(feature = "server")]
    pub fn new() -> ModelStore {
        let mut models =
            vec![
                Model {
                    index: ModelIndex(0),
                    name: "Engine Mk1".to_string(),
                    factory: Box::new(move |_| EngineModule::new(ModelIndex(0))),
                    shape: ModuleShape::new(vec![vec![b'#', b'#'],
                                                 vec![b'.', b'.']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(1),
                    name: "Command Mk1".to_string(),
                    factory: Box::new(move |_| CommandModule::new(ModelIndex(1))),
                    shape: ModuleShape::new(vec![vec![b'#', b'.'],
                                                 vec![b'#', b'.']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 4,
                },
                Model {
                    index: ModelIndex(2),
                    name: "Solar Mk1".to_string(),
                    factory: Box::new(move |_| SolarModule::new(ModelIndex(2))),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(3),
                    name: "Shield Mk1".to_string(),
                    factory: Box::new(move |_| ShieldModule::new(ModelIndex(3))),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(4),
                    name: "Blaster Mk1".to_string(),
                    factory: Box::new(move |_| ProjectileWeaponModule::new(ModelIndex(4))),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(5),
                    name: "Beam Mk1".to_string(),
                    factory: Box::new(move |_| BeamWeaponModule::new(ModelIndex(5))),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(6),
                    name: "Repair Mk1".to_string(),
                    factory: Box::new(move |_| RepairModule::new(ModelIndex(6))),
                    shape: ModuleShape::new(vec![vec![b'#']]),
                    power: 2,
                    min_hp: 2,
                    max_hp: 3,
                },
                Model {
                    index: ModelIndex(7),
                    name: "Cabin Mk1".to_string(),
                    factory: Box::new(move |_| CabinModule::new(ModelIndex(7))),
                    shape: ModuleShape::new(vec![vec![b'#', b'.', b'.'],
                                                 vec![b'#', b'#', b'.'],
                                                 vec![b'#', b'.', b'.']]),
                    power: 0,
                    min_hp: 3,
                    max_hp: 8,
                },
            ];
        
        let mut model_store = ModelStore { models: models };

        // Read module models from text files
        let paths = fs::read_dir("resources/data/modules").unwrap();
        for path in paths.map(|p| p.unwrap().path()) {
            if path.is_file() {
                model_store.add_model_from_properties(&config::read_properties(BufReader::new(File::open(&path).unwrap())));
            }
        }
    
        model_store
    }
    
    #[cfg(feature = "client")]
    fn add_model_from_properties(&mut self, ctx: &mut Context, prop: &HashMap<String, String>) {
        let factory = factory_from_properties(prop);
        
        let shape: Vec<Vec<u8>> =
            prop["shape"].lines()
                         .map(|l| l.trim_left().trim_right().bytes().collect())
                         .collect();
        
        let index = ModelIndex(self.models.len() as u16);
        
        self.models.push(Model {
            index: index,
            name: prop["name"].clone(),
            factory: factory,
            icon: Image::new(ctx, prop["icon"].as_str()).unwrap(),
            shape: ModuleShape::new(shape),
            power: prop["power"].parse().unwrap(),
            min_hp: prop["min_hp"].parse().unwrap(),
            max_hp: prop["max_hp"].parse().unwrap(),
        });
    }

    #[cfg(feature = "server")]
    fn add_model_from_properties(&mut self, prop: &HashMap<String, String>) {
        let factory = factory_from_properties(prop);
        
        let shape: Vec<Vec<u8>> =
            prop["shape"].lines()
                         .map(|l| l.trim_right()) // Trim newline
                         .filter(|l| l.len() > 0)
                         .map(|l| l.bytes().collect())
                         .collect();
        
        let index = ModelIndex(self.models.len() as u16);
        
        self.models.push(Model {
            index: index,
            name: prop["name"].clone(),
            factory: factory,
            shape: ModuleShape::new(shape),
            power: prop["power"].parse().unwrap(),
            min_hp: prop["min_hp"].parse().unwrap(),
            max_hp: prop["max_hp"].parse().unwrap(),
        });
    }
    
    pub fn models(&self) -> &Vec<Model> {
        &self.models
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelIndex(pub u16);

impl ModelIndex {
    pub fn get<'a>(self, model_store: &'a ModelStore) -> &'a Model {
        &model_store.models[self.0 as usize]
    }
}

fn factory_from_properties(prop: &HashMap<String, String>)
    -> Box<Fn(&Model) -> Module + Sync + Send> 
{
    let module_class = prop["class"].as_str();
    let prop_cloned = prop.clone();
    let factory: Box<Fn(&Model) -> Module + Sync + Send> =
        match module_class {
            "ProjectileWeapon" => {
                Box::new(move |model| {
                    ProjectileWeaponModule::from_properties(model, &prop_cloned)
                })
            },
            "BeamWeapon" => {
                Box::new(move |model| {
                    BeamWeaponModule::from_properties(model, &prop_cloned)
                })
            },
            "Command" => {
                Box::new(move |_| {
                    CommandModule::new(ModelIndex(1))
                })
            },
            "Cabin" => {
                Box::new(move |_| {
                    CabinModule::new(ModelIndex(7))
                })
            },
            "Shield" => {
                Box::new(move |_| {
                    ShieldModule::new(ModelIndex(3))
                })
            },
            "Solar" => {
                Box::new(move |_| {
                    SolarModule::new(ModelIndex(2))
                })
            },
            "Repair" => {
                Box::new(move |_| {
                    RepairModule::new(ModelIndex(6))
                })
            },
            "Engine" => {
                Box::new(move |_| {
                    EngineModule::new(ModelIndex(0))
                })
            },
            _ => {
                panic!("Unknown module class: {}", module_class);
            },
        };
    
    factory
}
