use net::{ServerSlot, Client, Joined, ReceivedPacket};

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server

pub struct ServerBattleState {
    slot: Box<ServerSlot>,
}

impl ServerBattleState {
    pub fn new(slot: Box<ServerSlot>) -> ServerBattleState {
        ServerBattleState{slot: slot}
    }
    
    pub fn run(&mut self) {
        loop {
            match self.slot.receive() {
                Joined(client_id) => {
                    println!("Client {} joined battle {}", client_id, self.slot.id());
                },
                ReceivedPacket(client_id, packet) => {
                    println!("Battle {} received packet from {} of length {}", self.slot.id(), client_id, packet.len());
                },
                _ => {}
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Client

pub struct ClientBattleState {
    client: Client,
}