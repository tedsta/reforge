use std::ops::{Deref, DerefMut};
use std::rand::Rng;
use std::rand;
use std::any::TypeId;

use ship::{Ship, ShipRef};
use module;
use module::{IModule, EngineModule, ProjectileWeaponModule, ShieldModule};

pub fn run_ai(ship: &mut Ship, enemy_ships: &Vec<ShipRef>) {
    // Random number generater
    let mut rng = rand::thread_rng();

    // Activate stuff, notice order of priority
    let mut activating_stuff = true;
    while activating_stuff {
        activating_stuff = false;
        // Weapon
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            let module_borrowed = module.borrow();
            if module_borrowed.get_type_id() == TypeId::of::<ProjectileWeaponModule>() {
                if !module_borrowed.get_base().plan_powered && ship.state.can_plan_activate_module(module_borrowed.get_base()) {
                    module_to_activate = Some(module.clone());
                    activating_stuff = true;
                    break;
                }
            }
        }
        if let Some(module) = module_to_activate {
            ship.state.activate_module(module.borrow_mut().get_base_mut());
        }
        // Engine
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            let module_borrowed = module.borrow();
            if module_borrowed.get_type_id() == TypeId::of::<EngineModule>() {
                if !module_borrowed.get_base().plan_powered && ship.state.can_plan_activate_module(module_borrowed.get_base()) {
                    module_to_activate = Some(module.clone());
                    activating_stuff = true;
                    break;
                }
            }
        }
        if let Some(module) = module_to_activate {
            ship.state.activate_module(module.borrow_mut().get_base_mut());
        }
        // Shield
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            let module_borrowed = module.borrow();
            if module_borrowed.get_type_id() == TypeId::of::<ShieldModule>() {
                if !module_borrowed.get_base().plan_powered && ship.state.can_plan_activate_module(module_borrowed.get_base()) {
                    module_to_activate = Some(module.clone());
                    activating_stuff = true;
                    break;
                }
            }
        }
        if let Some(module) = module_to_activate {
            ship.state.activate_module(module.borrow_mut().get_base_mut());
        }
    }
    
    // Try to target weapons
    if !enemy_ships.is_empty() {
        for module in ship.modules.iter() {
            let mut module_borrowed = module.borrow_mut();
            if module_borrowed.get_type_id() == TypeId::of::<ProjectileWeaponModule>() {
                if module_borrowed.get_base().is_active() {
                    let target_ship = &enemy_ships[rng.gen::<usize>() % enemy_ships.len()];
                    let target_module = &target_ship.borrow().modules[rng.gen::<usize>() % target_ship.borrow().modules.len()];
                
                    module_borrowed.get_base_mut().plan_target =
                        Some(module::Target {
                            ship: target_ship.clone(),
                            data: module::TargetData::TargetModule(target_module.clone()),
                        });
                }
            }
        }
    }
}
