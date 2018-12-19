use module::{ModelIndex, ModuleIndex};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ShipEditAction {
    Place(ModelIndex, u8, u8),
    Remove(ModuleIndex),
}