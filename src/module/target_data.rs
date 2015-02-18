use battle_state::BattleContext;
use ship::{ShipId, ShipRef};
use super::ModuleRef;

#[derive(PartialEq, RustcEncodable, RustcDecodable)]
pub enum TargetMode {
    TargetShip,
    TargetModule,
    OwnModule,
    AnyModule,
    Beam,
}

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub enum TargetData {
    TargetShip(ShipRef),
    TargetModule(ShipRef, ModuleRef),
    OwnModule(ShipRef, ModuleRef),
    AnyModule(ShipRef, ModuleRef),
    Beam,
}

// Target data suitable for sending over the network
#[derive(RustcEncodable, RustcDecodable)]
pub enum NetworkTargetData {
    TargetShip(ShipId),
    TargetModule(ShipId, u32),
    OwnModule(ShipId, u32),
    AnyModule(ShipId, u32),
    Beam,
}

impl NetworkTargetData {
    pub fn from_target_data(target_data: &TargetData) -> NetworkTargetData {
        use self::TargetData::*;
    
        match target_data {
            &TargetShip(ref ship) => NetworkTargetData::TargetShip(ship.borrow().id),
            &TargetModule(ref ship, ref module) => NetworkTargetData::TargetModule(ship.borrow().id, module.borrow().get_base().index),
            &OwnModule(ref ship, ref module) => NetworkTargetData::OwnModule(ship.borrow().id, module.borrow().get_base().index),
            &AnyModule(ref ship, ref module) => NetworkTargetData::AnyModule(ship.borrow().id, module.borrow().get_base().index),
            &Beam => NetworkTargetData::Beam,
        }
    }
    
    pub fn to_target_data(&self, context: &BattleContext) -> TargetData {
        match self {
            &NetworkTargetData::TargetShip(ref ship_id) => {
                let ship = context.get_ship(*ship_id);

                TargetData::TargetShip(ship.clone())
            },
            &NetworkTargetData::TargetModule(ref ship_id, ref module_index) => {
                let ship = context.get_ship(*ship_id);
                let module = ship.borrow().modules[(*module_index) as usize].clone();
                
                TargetData::TargetModule(ship.clone(), module.clone())
            },
            &NetworkTargetData::OwnModule(ref ship_id, ref module_index) => {
                let ship = context.get_ship(*ship_id);
                let module = ship.borrow().modules[(*module_index) as usize].clone();
                
                TargetData::TargetModule(ship.clone(), module.clone())
            },
            &NetworkTargetData::AnyModule(ref ship_id, ref module_index) => {
                let ship = context.get_ship(*ship_id);
                let module = ship.borrow().modules[(*module_index) as usize].clone();
                
                TargetData::TargetModule(ship.clone(), module.clone())
            },
            &NetworkTargetData::Beam => {
                TargetData::Beam
            },
        }
    }
}