use std::ops::DerefMut;

use module::{ModuleBox, ModuleRef};
use ship::ShipRef;
use sim::SimEvent;

pub struct DamageEvent {
    ship: ShipRef,
    module: ModuleRef,
    damage: u8,
}

impl DamageEvent {
    pub fn new(ship: ShipRef, module: ModuleRef, damage: u8) -> DamageEvent {
        DamageEvent {
            ship: ship,
            module: module,
            damage: damage,
        }
    }
}

impl SimEvent for DamageEvent {
    fn apply(&mut self, module: &mut ModuleBox) {
        let mut ship = self.ship.borrow_mut();
        ship.deal_damage(self.module.borrow_mut().deref_mut(), self.damage);
    }
}