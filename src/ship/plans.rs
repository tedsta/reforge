use module::ModulePlans;
use sector_data::SectorId;

#[derive(RustcEncodable, RustcDecodable)]
pub struct ShipPlans {
    pub logout: bool,
    pub target_sector: Option<SectorId>,
    pub modules: Vec<ModulePlans>,
}