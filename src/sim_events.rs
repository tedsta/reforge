use std::ops::DerefMut;

use module::{ModuleBox, ModuleRef};
use ship::{ShipRef, ShipState};
use sim::SimEvent;

pub struct DamageEvent {
    module: ModuleRef,
    damage: u8,
}

impl DamageEvent {
    pub fn new(module: ModuleRef, damage: u8) -> DamageEvent {
        DamageEvent {
            module: module,
            damage: damage,
        }
    }
}

impl SimEvent for DamageEvent {
    fn apply(&mut self, ship_state: &mut ShipState) {
        ship_state.deal_damage(self.module.borrow_mut().deref_mut(), self.damage);
    }
}