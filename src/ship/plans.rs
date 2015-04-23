use module::ModulePlans;
use sector_data::SectorId;

#[derive(RustcEncodable, RustcDecodable)]
pub struct ShipPlans {
    pub logout: bool,
    pub target_sector: Option<SectorId>,
    pub modules: Vec<ModulePlans>,
}

impl ShipPlans {
    pub fn new() -> ShipPlans {
        ShipPlans {
            logout: false,
            target_sector: None,
            modules: vec!(),
        }
    }
}