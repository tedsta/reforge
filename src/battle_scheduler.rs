use net::{ServerSlot, Joined, ReceivedPacket};

pub struct BattleScheduler {
    waiting: Vec<u32>, // Clients still waiting for a battle
}

impl BattleScheduler {
    pub fn new() -> BattleScheduler {
        BattleScheduler{waiting: vec!()}
    }

    pub fn run(&mut self, slot: Box<ServerSlot>) {
        loop {
            match slot.receive() {
                Joined(client_id) => {
                    println!("Client {} joined the scheduler", client_id);
                    self.waiting.push(client_id);
                },
                ReceivedPacket(client_id, packet) => {
                    println!("Received packet from {} of length {}", client_id, packet.len());
                },
                _ => {}
            }
        }
    }
}