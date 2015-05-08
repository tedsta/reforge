use super::{
    Module,
    
    ProjectileWeaponModule,
};

pub struct Model {
    pub name: String,
    pub factory: Box<Fn() -> Module + Sync + Send>,
}

pub struct ModelStore {
    models: Vec<Model>,
}

impl ModelStore {
    pub fn new() -> ModelStore {
        let models =
            vec![
                Model {
                    name: "Blaster Mk1".to_string(),
                    factory: Box::new(move || ProjectileWeaponModule::new()),
                },
            ];
    
        ModelStore {
            models: models,
        }
    }
}

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct ModelIndex(u16);

impl ModelIndex {
    pub fn get<'a>(self, model_store: &'a ModelStore) -> &'a Model {
        &model_store.models[self.0 as usize]
    }
}