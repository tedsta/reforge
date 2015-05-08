use super::{
    Module,
    
    ProjectileWeaponModule,
};

pub struct Model {
    pub name: String,
    factory: Box<Fn() -> Module + Sync + Send>,
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