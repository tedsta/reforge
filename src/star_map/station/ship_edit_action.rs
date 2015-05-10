use module::{ModelIndex, ModuleIndex};

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum ShipEditAction {
    Place(ModelIndex, u8, u8),
    Remove(ModuleIndex),
}