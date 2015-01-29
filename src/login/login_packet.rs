#[derive(RustcEncodable, RustcDecodable)]
pub struct LoginPacket {
    pub username: String,
    pub password: String,
}