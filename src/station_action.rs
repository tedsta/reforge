use sector_data::SectorId;

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum StationAction {
    Jump(SectorId),
}