use ship::Ship;
use module::{EngineModule, ProjectileWeaponModule};

pub fn generate_ship() -> Ship {
    let mut ship = Ship::new();
    
    let mut weapon = ProjectileWeaponModule::new();
    weapon.get_base().x = 1;
    weapon.get_base().y = 1;
    
    let mut engine = EngineModule::new();
    engine.get_base().x = 1;
    engine.get_base().y = 2;
    
    ship.add_module(weapon);
    ship.add_module(engine);
    ship
}