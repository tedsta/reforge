use chat::ChatMsg;

// Packets sent from client to server
#[derive(RustcEncodable, RustcDecodable)]
pub enum ServerBattlePacket {
    Plan,
    Chat(String),
    Logout,
}

// Packets sent from server to client
#[derive(RustcEncodable, RustcDecodable)]
pub enum ClientBattlePacket {
    NewShipsPre,
    SimResults,
    NewShipsPost,
    Tick(bool), // Tick and whether it's the last
    Chat(ChatMsg),
}