use module::{Module, ModuleIndex, ModulePlans};
use sector_data::SectorId;
use ship::{Ship, ShipIndex, ShipState};

#[derive(Clone, RustcEncodable, RustcDecodable)]
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
    
    pub fn can_plan_activate_module(&self, ship_state: &ShipState, module: &Module) -> bool {
        module.can_activate()
            && !self.module_plans[module.index.to_usize()].active
            && self.available_plan_power(ship_state) >= module.get_power()
    }
    
    pub fn plan_activate_module(&mut self, module: &Module) {
        self.plan_power_use += module.get_power();
        self.module_plans.get_mut(module.index.to_usize())
            .expect("Failed to plan activate non-existant module")
            .active = true;
    }
    
    pub fn plan_deactivate_module(&mut self, module: &Module) {
        self.plan_power_use -= module.get_power();
        self.module_plans.get_mut(module.index.to_usize())
            .expect("Failed to plan activate non-existant module")
            .active = false;
    }
    
    pub fn deactivate_unpowerable_modules(&mut self, ship: &Ship) {
        for module in &ship.modules {
            if self.plan_power_use <= ship.state.max_power {
                break;
            } else {
                if module.get_power() > 0 {
                    if !module.active && self.module_plans[module.index.to_usize()].active {
                        self.plan_deactivate_module(module);
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