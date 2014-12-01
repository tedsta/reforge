use ship::{Ship, ShipId};
use module::{EngineModule, ProjectileWeaponModule, ShieldModule, SolarModule, ModuleType, ModuleTypeStore};

pub fn generate_ship(mod_store: &ModuleTypeStore, id: ShipId) -> Ship {
    let mut ship = Ship::new(id);
    
    let mut engine = EngineModule::new(mod_store, 0);
    engine.get_base_mut().x = 0;
    engine.get_base_mut().y = 0;
    
    let mut weapon1 = ProjectileWeaponModule::new(mod_store, 1);
    weapon1.get_base_mut().x = 3;
    weapon1.get_base_mut().y = 1;
    
    let mut weapon2 = ProjectileWeaponModule::new(mod_store, 1);
    weapon2.get_base_mut().x = 3;
    weapon2.get_base_mut().y = 2;
    
    let mut shield1 = ShieldModule::new(mod_store, 2);
    shield1.get_base_mut().x = 2;
    shield1.get_base_mut().y = 1;
    
    let mut shield2 = ShieldModule::new(mod_store, 2);
    shield2.get_base_mut().x = 2;
    shield2.get_base_mut().y = 2;
    
    let mut solar1 = SolarModule::new(mod_store, 3);
    solar1.get_base_mut().x = 1;
    solar1.get_base_mut().y = 1;
    
    let mut solar2 = SolarModule::new(mod_store, 3);
    solar2.get_base_mut().x = 1;
    solar2.get_base_mut().y = 2;
    
    ship.add_module(engine);
    ship.add_module(weapon1);
    ship.add_module(weapon2);
    ship.add_module(shield1);
    ship.add_module(shield2);
    ship.add_module(solar1);
    ship.add_module(solar2);
    ship
}