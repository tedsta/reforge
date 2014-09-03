// Packets sent from client to server
#[deriving(FromPrimitive)]
pub enum ServerPacketId {
    Plan, // Player's plans
}

// Packets sent from server to client
#[deriving(FromPrimitive)]
pub enum ClientPacketId {
    SimResults, // Calculated simulation results from server
}