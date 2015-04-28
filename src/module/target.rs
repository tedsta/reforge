use battle_context::BattleContext;
use ship::{Ship, ShipIndex};
use vec::{Vec2, Vec2f};

use super::{ModuleRef, ModuleIndex};

#[derive(Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum TargetMode {
    TargetShip,
    TargetModule,
    OwnModule,
    AnyModule,
    Beam(u8),
}

#[derive(Clone, Copy, RustcEncodable, RustcDecodable)]
pub struct Target {
    pub ship: ShipIndex,
    pub data: TargetData,
}

#[derive(Clone, Copy, RustcEncodable, RustcDecodable)]
pub enum TargetData {
    TargetShip,
    TargetModule(ModuleIndex),
    OwnModule(ModuleIndex),
    AnyModule(ModuleIndex),
    Beam(Vec2f, Vec2f),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Manifestation of a target
pub struct TargetManifest<'a> {
    pub ship: &'a Ship,
    pub data: TargetManifestData,
}

impl<'a> TargetManifest<'a> {
    pub fn from_target(bc: &'a BattleContext, target: &Target) -> TargetManifest<'a> {
        let ship = target.ship.get(bc);
        let data = TargetManifestData::from_target_data(ship, &target.data);
        
        TargetManifest {
            ship: ship,
            data: data,
        }
    }
}

pub enum TargetManifestData {
    TargetShip,
    TargetModule(ModuleRef),
    OwnModule(ModuleRef),
    AnyModule(ModuleRef),
    Beam(Vec2f, Vec2f),
}

impl TargetManifestData {
    fn from_target_data(ship: &Ship, target_data: &TargetData) -> TargetManifestData {
        match target_data {
            &TargetData::TargetShip => TargetManifestData::TargetShip,
            &TargetData::TargetModule(module_index) => {
                TargetManifestData::TargetModule(ship.modules[module_index.to_usize()].clone())
            },
            &TargetData::OwnModule(module_index) => {
                TargetManifestData::TargetModule(ship.modules[module_index.to_usize()].clone())
            },
            &TargetData::AnyModule(module_index) => {
                TargetManifestData::TargetModule(ship.modules[module_index.to_usize()].clone())
            },
            &TargetData::Beam(start, end) => TargetManifestData::Beam(start, end),
        }
    }
}