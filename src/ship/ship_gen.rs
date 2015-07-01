use rand::Rng;
use rand;
use std::cmp;

use ship::{Ship, ShipId};
use module::{
    ModelIndex,
    ModelStore,
    
    Module,
    ModuleShape,

    EngineModule,
    ProjectileWeaponModule,
    ShieldModule,
    SolarModule,
    CommandModule,
    BeamWeaponModule
};

pub fn generate_ship(id: ShipId, name: String, level: u8) -> Ship {
    if level == 0 {
        panic!("Can't generate ship with level 0");
    }

    // Random number generater
    let mut rng = rand::thread_rng();

    // Brand new ship!!
    let mut ship = Ship::new(id, name, level);
    
    // Generate ship height
    let height = cmp::min(8, rng.gen::<u8>()%(cmp::max(level, 2)) + cmp::max(1, level/2));

    // Generate some random module counts
    let mut num_power = cmp::max(height, rng.gen::<u8>()%(level + 1) + 1);
    let num_engines = cmp::min(height, (rng.gen::<u8>()%(level + 1))/2 + 1);
    let num_shields = rng.gen::<u8>()%(level + 1);
    let num_weapons = rng.gen::<u8>()%(level + 1) + 1;
    let num_beams = rng.gen::<u8>()%(level/2) + 1;
    
    // Add top half engines
    for i in 0 .. num_engines/2 + num_engines%2 {
        let mut engine = EngineModule::new(ModelIndex(0));
        engine.x = 0;
        engine.y = i;
        ship.add_module(engine);
    }
    
    // Add bottom half engines
    for i in 0 .. num_engines/2 {
        let mut engine = EngineModule::new(ModelIndex(0));
        engine.x = 0;
        engine.y = height - 1 - i;
        ship.add_module(engine);
    }
    
    // Fill in any remaining space between engines with power modules
    for i in 0 .. height - num_engines {
        let mut solar = SolarModule::new(ModelIndex(2));
        solar.x = 1;
        solar.y = num_engines/2 + num_engines%2 + i;
        ship.add_module(solar);
        num_power -= 1;
    }
    
    // Now, randomly fill up rest of ship with remaining modules
    let mut x = 2;
    let mut y = 0;
    
    let mut module_counts = [num_power, num_shields, num_weapons, num_beams];
    
    // While there's still modules to be placed...
    while module_counts.iter().filter(|x| **x > 0).count() > 0 {
        // Choose a module type
        let mut choice = rng.gen::<u8>()%(module_counts.len() as u8);
        
        // Make sure there are modules left to place of that type
        while module_counts[choice as usize] == 0 {
            choice += 1;
            if choice as usize >= module_counts.len() {
                choice = 0;
            }
        }
        
        // Power module
        if choice == 0 {
            let mut solar = SolarModule::new(ModelIndex(2));
            solar.x = x;
            solar.y = y;
            ship.add_module(solar);
        } else if choice == 1 {
            let mut shield = ShieldModule::new(ModelIndex(3));
            shield.x = x;
            shield.y = y;
            ship.add_module(shield);
        } else if choice == 2 {
            let mut weapon = ProjectileWeaponModule::new(ModelIndex(4));
            weapon.x = x;
            weapon.y = y;
            ship.add_module(weapon);
        } else if choice == 3 {
            let mut beam = BeamWeaponModule::new(ModelIndex(5));
            beam.x = x;
            beam.y = y;
            ship.add_module(beam);
        }
        
        // Decrement the chosen module's pool
        module_counts[choice as usize] -= 1;
    
        // Move cursor
        y += 1;
        if y >= height {
            y = 0;
            x += 1;
        }
    }
    
    // Figure out where to put command module
    let mut command_x = ship.get_width();
    let command_y = cmp::min(height - 1, rng.gen::<u8>()%(height + 1));
    
    while ship.is_space_free(command_x - 1, command_y, &ModuleShape::new(vec![vec![b'#', b'.'], vec![b'#', b'.']])) {
        command_x -= 1;
    }
    
    // Finally, add the command module
    let mut command = CommandModule::new(ModelIndex(1));
    command.x = command_x;
    command.y = command_y;
    ship.add_module(command);
    
    ship
}

pub fn generate_dummy_ship(id: ShipId, name: String) -> Ship {
    // Random number generater
    let mut rng = rand::thread_rng();

    // Brand new ship!!
    let mut ship = Ship::new(id, name, 1);
    
    // Generate ship height
    let height = rng.gen::<u8>()%4 + 1;

    // Generate some random module counts
    let mut num_power = rng.gen::<u8>()%3+2;
    let num_engines = cmp::min(height, rng.gen::<u8>()%5 + 1);
    let num_shields = rng.gen::<u8>()%5 + 1;
    let num_weapons = 0;
    let num_beams = 0;
    
    // Add top half engines
    for i in 0 .. num_engines/2 + num_engines%2 {
        let mut engine = EngineModule::new(ModelIndex(0));
        engine.x = 0;
        engine.y = i;
        ship.add_module(engine);
    }
    
    // Add bottom half engines
    for i in 0 .. num_engines/2 {
        let mut engine = EngineModule::new(ModelIndex(0));
        engine.x = 0;
        engine.y = height - 1 - i;
        ship.add_module(engine);
    }
    
    // Fill in any remaining space between engines with power modules
    for i in 0 .. height - num_engines {
        let mut solar = SolarModule::new(ModelIndex(2));
        solar.x = 1;
        solar.y = num_engines/2 + num_engines%2 + i;
        ship.add_module(solar);
        num_power -= 1;
    }
    
    // Now, randomly fill up rest of ship with remaining modules
    let mut x = 2;
    let mut y = 0;
    
    let mut module_counts = [num_power, num_shields, num_weapons, num_beams];
    
    // While there's still modules to be placed...
    while module_counts.iter().filter(|x| **x > 0).count() > 0 {
        // Choose a module type
        let mut choice = rng.gen::<u8>()%(module_counts.len() as u8);
        
        // Make sure there are modules left to place of that type
        while module_counts[choice as usize] == 0 {
            choice += 1;
            if choice as usize >= module_counts.len() {
                choice = 0;
            }
        }
        
        // Power module
        if choice == 0 {
            let mut solar = SolarModule::new(ModelIndex(2));
            solar.x = x;
            solar.y = y;
            ship.add_module(solar);
        } else if choice == 1 {
            let mut shield = ShieldModule::new(ModelIndex(3));
            shield.x = x;
            shield.y = y;
            ship.add_module(shield);
        } else if choice == 2 {
            let mut weapon = ProjectileWeaponModule::new(ModelIndex(4));
            weapon.x = x;
            weapon.y = y;
            ship.add_module(weapon);
        } else if choice == 3 {
            let mut beam = BeamWeaponModule::new(ModelIndex(5));
            beam.x = x;
            beam.y = y;
            ship.add_module(beam);
        }
        
        // Decrement the chosen module's pool
        module_counts[choice as usize] -= 1;
    
        // Move cursor
        y += 1;
        if y >= height {
            y = 0;
            x += 1;
        }
    }
    
    // Figure out where to put command module
    let mut command_x = ship.get_width();
    let command_y = cmp::min(height - 1, rng.gen::<u8>()%(height + 1));
    
    while ship.is_space_free(command_x - 1, command_y, &ModuleShape::new(vec![vec![b'#', b'.'], vec![b'#', b'.']])) {
        command_x -= 1;
    }
    
    // Finally, add the command module
    let mut command = CommandModule::new(ModelIndex(1));
    command.x = command_x;
    command.y = command_y;
    ship.add_module(command);
    
    ship
}

pub fn generate_dev_ship(model_store: &ModelStore, id: ShipId, name: String) -> Ship {
    // Random number generater
    let mut rng = rand::thread_rng();

    // Brand new ship!!
    let mut ship = Ship::new(id, name, 1);
    
    // Generate ship height
    let height = 8;

    // Generate some random module counts
    let mut num_power = 15;
    let num_engines = 7;
    let num_shields = 15;
    
    // Add top half engines
    for i in 0 .. num_engines/2 + num_engines%2 {
        let mut engine = EngineModule::new(ModelIndex(0));
        engine.x = 0;
        engine.y = i;
        ship.add_module(engine);
    }
    
    // Add bottom half engines
    for i in 0 .. num_engines/2 {
        let mut engine = EngineModule::new(ModelIndex(0));
        engine.x = 0;
        engine.y = height - 1 - i;
        ship.add_module(engine);
    }
    
    // Fill in any remaining space between engines with power modules
    for i in 0 .. height - num_engines {
        let mut solar = SolarModule::new(ModelIndex(2));
        solar.x = 1;
        solar.y = num_engines/2 + num_engines%2 + i;
        ship.add_module(solar);
        num_power -= 1;
    }
    
    // Now, randomly fill up rest of ship with remaining modules
    let mut x = 2;
    let mut y = 0;
    
    let mut module_counts = [num_power, num_shields];
    
    // While there's still modules to be placed...
    while module_counts.iter().filter(|x| **x > 0).count() > 0 {
        // Choose a module type
        let mut choice = rng.gen::<usize>()%module_counts.len();
        
        // Make sure there are modules left to place of that type
        while module_counts[choice] == 0 {
            choice += 1;
            if choice >= module_counts.len() {
                choice = 0;
            }
        }
        
        // Power module
        if choice == 0 {
            let mut solar = SolarModule::new(ModelIndex(2));
            solar.x = x;
            solar.y = y;
            ship.add_module(solar);
        } else if choice == 1 {
            let mut shield = ShieldModule::new(ModelIndex(3));
            shield.x = x;
            shield.y = y;
            ship.add_module(shield);
        }
        
        // Decrement the chosen module's pool
        module_counts[choice] -= 1;
    
        // Move cursor
        y += 1;
        if y >= height {
            y = 0;
            x += 1;
        }
    }
    
    let mut modules: Vec<(ModelIndex, u8)> = model_store.models().iter().skip(4).map(|m| (m.index, 1)).collect();
    
    // While there's still modules to be placed...
    while modules.iter().filter(|m| m.1 > 0).count() > 0 {
        // Choose a module type
        let mut choice = rng.gen::<usize>()%modules.len();
        
        // Make sure there are modules left to place of that type
        while modules[choice].1 == 0 {
            choice += 1;
            if choice >= modules.len() {
                choice = 0;
            }
        }
        
        let mut module = modules[choice].0.get(model_store).create();
        module.x = x;
        module.y = y;
        
        // Decrement the chosen module's pool
        modules[choice].1 -= 1;
    
        // Move cursor
        y += module.shape.side();
        if y >= height {
            y = 0;
            x += 1;
        }
        
        ship.add_module(module);
    }
    
    // Figure out where to put command module
    let mut command_x = ship.get_width();
    let command_y = cmp::min(height - 1, rng.gen::<u8>()%(height + 1));
    
    while ship.is_space_free(command_x - 1, command_y, &ModuleShape::new(vec![vec![b'#', b'.'], vec![b'#', b'.']])) {
        command_x -= 1;
    }
    
    // Finally, add the command module
    let mut command = CommandModule::new(ModelIndex(1));
    command.x = command_x;
    command.y = command_y;
    ship.add_module(command);
    
    ship
}
