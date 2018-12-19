#[derive(Serialize, Deserialize)]
pub struct LoginPacket {
    pub username: String,
    pub password: String,
}