use battle_state::BattleContext;
use ship::{ShipId, ShipRef};
use vec::{Vec2, Vec2f};

use super::ModuleRef;

#[derive(Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum TargetMode {
    TargetShip,
    TargetModule,
    OwnModule,
    AnyModule,
    Beam(u8),
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Target {
    pub ship: ShipRef,
    pub data: TargetData,
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum TargetData {
    TargetShip,
    TargetModule(ModuleRef),
    OwnModule(ModuleRef),
    AnyModule(ModuleRef),
    Beam(Vec2f, Vec2f),
}

// Target data suitable for sending over the network
#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct NetworkTarget {
    pub ship: ShipId,
    pub data: NetworkTargetData,
}

impl NetworkTarget {
    pub fn from_target(target: &Target) -> NetworkTarget {
        use self::TargetData::*;
    
        NetworkTarget {
            ship: target.ship.borrow().id,
            data: NetworkTargetData::from_target_data(&target.data),
        }
    }
    
    pub fn to_target(&self, context: &BattleContext) -> Target {
        let ship = context.get_ship(self.ship);
    
        Target {
            ship: ship.clone(),
            data: self.data.to_target_data(ship),
        }
    }
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum NetworkTargetData {
    TargetShip,
    TargetModule(u32),
    OwnModule(u32),
    AnyModule(u32),
    Beam(Vec2f, Vec2f),
}

impl NetworkTargetData {
    pub fn from_target_data(target_data: &TargetData) -> NetworkTargetData {
        use self::TargetData::*;
    
        match target_data {
            &TargetShip => NetworkTargetData::TargetShip,
            &TargetModule(ref module) => NetworkTargetData::TargetModule(module.borrow().get_base().index),
            &OwnModule(ref module) => NetworkTargetData::OwnModule(module.borrow().get_base().index),
            &AnyModule(ref module) => NetworkTargetData::AnyModule(module.borrow().get_base().index),
            &Beam(start, end) => NetworkTargetData::Beam(start, end),
        }
    }
    
    pub fn to_target_data(&self, ship: &ShipRef) -> TargetData {
        match self {
            &NetworkTargetData::TargetShip => TargetData::TargetShip,
            &NetworkTargetData::TargetModule(ref module_index) => {
                TargetData::TargetModule(ship.borrow().modules[(*module_index) as usize].clone())
            },
            &NetworkTargetData::OwnModule(ref module_index) => {
                TargetData::TargetModule(ship.borrow().modules[(*module_index) as usize].clone())
            },
            &NetworkTargetData::AnyModule(ref module_index) => {
                TargetData::TargetModule(ship.borrow().modules[(*module_index) as usize].clone())
            },
            &NetworkTargetData::Beam(start, end) => TargetData::Beam(start, end),
        }
    }
}