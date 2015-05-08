use module::{ModelIndex, ModuleIndex};

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum ShipEditAction {
    Place(ModelIndex),
    Remove(ModuleIndex),
}