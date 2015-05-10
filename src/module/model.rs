use super::{
    ModuleStored,
    
    EngineModule,
};

pub struct Model {
    pub name: String,
    factory: Box<Fn() -> ModuleStored + Sync + Send>,
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