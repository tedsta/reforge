use ship::Ship;
use module::{EngineModule};

pub fn generate_ship() -> Ship {
    let mut ship = Ship::new();
    
    let mut engine = EngineModule::new();
    engine.get_base().x = 1;
    engine.get_base().y = 2;
    
    ship.add_module(engine);
    ship
}