use sector_data::SectorId;

use super::ShipEditAction;

#[derive(Clone, Serialize, Deserialize)]
pub enum StationAction {
    Jump(SectorId),
    ShipEdit(ShipEditAction),
    Chat(String),
    Logout,
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