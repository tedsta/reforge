use std::ops::{Deref, DerefMut};
use rand::Rng;
use rand;
use std::any::TypeId;

use ship::{Ship, ShipPlans};
use module;
use module::{IModule, ModuleClass};

pub fn run_ai(ship: &Ship, plans: &mut ShipPlans, enemy_ships: &Vec<&Ship>) {
    // Random number generater
    let mut rng = rand::thread_rng();

    // Activate stuff, notice order of priority
    let mut activating_stuff = true;
    while activating_stuff {
        activating_stuff = false;
        // Weapon
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            if module.get_class() == ModuleClass::ProjectileWeapon {
                if !plans.module_plans(module.index).active && plans.can_plan_activate_module(&ship.state, module) {
                    module_to_activate = Some(module.index);
                    activating_stuff = true;
                    break;
                }
            }
        }
        if let Some(module) = module_to_activate {
            plans.plan_activate_module(module.get(ship));
        }
        // Engine
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            if module.get_class() == ModuleClass::Engine {
                if !plans.module_plans(module.index).active && plans.can_plan_activate_module(&ship.state, module) {
                    module_to_activate = Some(module.index);
                    activating_stuff = true;
                    break;
                }
            }
        }
        if let Some(module) = module_to_activate {
            plans.plan_activate_module(module.get(ship));
        }
        // Shield
        let mut module_to_activate = None;
        for module in ship.modules.iter() {
            if module.get_class() == ModuleClass::Shield {
                if !plans.module_plans(module.index).active && plans.can_plan_activate_module(&ship.state, module) {
                    module_to_activate = Some(module.index);
                    activating_stuff = true;
                    break;
                }
            }
        }
        if let Some(module) = module_to_activate {
            plans.plan_activate_module(module.get(ship));
        }
    }
    
    // Try to target weapons
    if !enemy_ships.is_empty() {
        for module in &ship.modules {
            if module.get_class() == ModuleClass::ProjectileWeapon {
                if module.active {
                    let target_ship = enemy_ships[rng.gen::<usize>() % enemy_ships.len()];
                    let target_module = &target_ship.modules[rng.gen::<usize>() % target_ship.modules.len()];
                
                    plans.module_plans(module.index).target =
                        Some(module::Target {
                            ship: target_ship.index,
                            data: module::TargetData::TargetModule(target_module.index),
                        });
                }
            }
        }
    }
}
