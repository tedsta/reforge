use sector_data::SectorId;

use super::ShipEditAction;

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum StationAction {
    Jump(SectorId),
    ShipEdit(ShipEditAction),
}

impl StationAction {
    pub fn is_jump(&self) -> bool {
        if let &StationAction::Jump(_) = self {
            true
        } else {
            false
        }
    }
}