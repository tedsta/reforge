use ship::{Ship, ShipId};
use module::{EngineModule, ProjectileWeaponModule};

pub fn generate_ship(id: u64) -> Ship {
    let mut ship = Ship::new(ShipId{id: id, index: None});
    
    let mut engine = EngineModule::new();
    engine.get_base_mut().x = 0;
    engine.get_base_mut().y = 0;
    
    let mut weapon = ProjectileWeaponModule::new();
    weapon.get_base_mut().x = 1;
    weapon.get_base_mut().y = 1;
    
    ship.add_module(engine);
    ship.add_module(weapon);
    ship
}