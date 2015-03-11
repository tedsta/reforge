use vec::Vec2f;

#[derive(Clone, Copy, PartialEq, Eq, Hash, RustcEncodable, RustcDecodable)]
pub struct SectorId(pub u32);

#[derive(Clone, Copy, RustcEncodable, RustcDecodable)]
pub struct SectorData {
    pub id: SectorId,
    pub map_position: Vec2f,
}