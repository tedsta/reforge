use vec::Vec2f;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SectorId(pub u32);

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SectorKind {
    Sector,
    Station,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct SectorData {
    pub id: SectorId,
    pub kind: SectorKind,
    pub map_position: Vec2f,
}