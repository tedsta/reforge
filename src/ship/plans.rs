use module::{ModuleBase, ModuleIndex, ModulePlans};
use sector_data::SectorId;
use ship::{Ship, ShipIndex, ShipState};

#[derive(RustcEncodable, RustcDecodable)]
pub struct ShipPlans {
    pub logout: bool,
    pub target_sector: Option<SectorId>,
    pub module_plans: Vec<ModulePlans>,
    
    pub plan_power_use: u8,
}

impl ShipPlans {    
    pub fn available_plan_power(&self, ship_state: &ShipState) -> u8 {
        if ship_state.max_power > self.plan_power_use {
            ship_state.max_power - self.plan_power_use
        } else {
            0
        }
    }
    
    pub fn module_plans(&mut self, index: ModuleIndex) -> &mut ModulePlans {
        &mut self.module_plans[index.to_usize()]
    }
    
    pub fn can_plan_activate_module(&self, ship_state: &ShipState, module: &ModuleBase) -> bool {
        module.can_activate()
            && !self.module_plans[module.index.to_usize()].plan_powered
            && self.available_plan_power(ship_state) >= module.get_power()
    }
    
    pub fn plan_activate_module(&mut self, module: &ModuleBase) {
        self.plan_power_use += module.get_power();
        self.module_plans.get_mut(module.index.to_usize())
            .expect("Failed to plan activate non-existant module")
            .plan_powered = true;
    }
    
    pub fn plan_deactivate_module(&mut self, module: &ModuleBase) {
        self.plan_power_use -= module.get_power();
        self.module_plans.get_mut(module.index.to_usize())
            .expect("Failed to plan activate non-existant module")
            .plan_powered = false;
    }
    
    pub fn deactivate_unpowerable_modules(&mut self, ship: &Ship) {
        for module in &ship.modules {
            if self.plan_power_use <= ship.state.max_power {
                break;
            } else {
                // Attempt to borrow the module
                if let Some(mut module) = module.try_borrow_mut() {
                    if module.get_base().get_power() > 0 {
                        if !module.get_base().powered && self.module_plans[module.get_base().index.to_usize()].plan_powered {
                            self.plan_deactivate_module(module.get_base_mut());
                        }
                    }
                }
            }
        }
    }
    
    pub fn on_ship_removed(&mut self, ship_index: ShipIndex) {
        for plans in &mut self.module_plans {
            plans.on_ship_removed(ship_index);
        }
    }
}