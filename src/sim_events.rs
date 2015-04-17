use std::ops::DerefMut;

use module::{ModuleBox, ModuleIndex, ModuleRef};
use ship::{ShipRef, ShipState};
use sim::SimEvent;

pub struct DamageEvent {
    module_index: ModuleIndex,
    damage: u8,
}

impl DamageEvent {
    pub fn new(module_index: ModuleIndex, damage: u8) -> DamageEvent {
        DamageEvent {
            module_index: module_index,
            damage: damage,
        }
    }
}

impl SimEvent for DamageEvent {
    fn apply(&mut self, ship_state: &mut ShipState) {
        ship_state.deal_damage(self.module_index, self.damage);
    }
}