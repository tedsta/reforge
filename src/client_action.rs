#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum ClientAction {
    JoinSector,
    JoinStation,
    Logout,
}