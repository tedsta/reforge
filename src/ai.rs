use std::rand::Rng;
use std::rand;

use ship::{Ship, ShipRef};
use module;
use module::IModule;

pub fn run_ai(ship: &mut Ship, enemy_ships: &Vec<ShipRef>) {
    use module::Module::*;
    
    // Random number generater
    let mut rng = rand::task_rng();

    // Activate stuff, notice order of priority
    let mut activating_stuff = true;
    while activating_stuff {
        activating_stuff = false;
        // Weapon
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            let module_borrowed = module.borrow();
            match *module_borrowed.deref() {
                ProjectileWeapon(_) => {
                    if !module_borrowed.get_base().plan_powered && ship.state.can_plan_activate_module(module_borrowed.get_base()) {
                        module_to_activate = Some(module.clone());
                        activating_stuff = true;
                        break;
                    }
                },
                _ => {},
            }
        }
        match module_to_activate {
            Some(module) => {
                ship.state.activate_module(module.borrow_mut().get_base_mut());
            },
            None => {},
        }
        // Engine
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            let module_borrowed = module.borrow();
            match *module_borrowed.deref() {
                Engine(_) => {
                    if !module_borrowed.get_base().plan_powered && ship.state.can_plan_activate_module(module_borrowed.get_base()) {
                        module_to_activate = Some(module.clone());
                        activating_stuff = true;
                        break;
                    }
                },
                _ => {},
            }
        }
        match module_to_activate {
            Some(module) => {
                ship.state.activate_module(module.borrow_mut().get_base_mut());
            },
            None => {},
        }
        // Shield
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            let module_borrowed = module.borrow();
            match *module_borrowed.deref() {
                Shield(_) => {
                    if !module_borrowed.get_base().plan_powered && ship.state.can_plan_activate_module(module_borrowed.get_base()) {
                        module_to_activate = Some(module.clone());
                        activating_stuff = true;
                        break;
                    }
                },
                _ => {},
            }
        }
        match module_to_activate {
            Some(module) => {
                ship.state.activate_module(module.borrow_mut().get_base_mut());
            },
            None => {},
        }
    }
    
    // Try to target weapons
    for module in ship.modules.iter() {
        let mut module_borrowed = module.borrow_mut();
        match module_borrowed.deref_mut() {
            &ProjectileWeapon(_) => {
                if module_borrowed.get_base().is_active() {
                    let target_ship = &enemy_ships[rng.gen::<uint>() % enemy_ships.len()];
                    let target_module = &target_ship.borrow().modules[rng.gen::<uint>() % target_ship.borrow().modules.len()];
                
                    module_borrowed.on_module_clicked(target_ship, target_module);
                }
            },
            _ => {},
        }
    }
}