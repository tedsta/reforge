use chat::ChatMsg;

// Packets sent from client to server
#[derive(Serialize, Deserialize)]
pub enum ServerBattlePacket {
    Plan,
    Chat(String),
    Logout,
}

// Packets sent from server to client
#[derive(Serialize, Deserialize)]
pub enum ClientBattlePacket {
    NewShipsPre,
    SimResults,
    NewShipsPost,
    Tick(Option<u8>), // Tick and whether it's the last
    Chat(ChatMsg),
}
