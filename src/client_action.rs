#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ClientAction {
    JoinSector,
    JoinStation,
    Logout,
}