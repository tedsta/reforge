use std::collections::RingBuf;

use net::{ClientId, ServerSlot, Joined, ReceivedPacket};
use battle_state::ServerBattleState;

pub struct BattleScheduler {
    slot: Box<ServerSlot>,
    waiting: RingBuf<ClientId>, // Clients still waiting for a battle
}

impl BattleScheduler {
    pub fn new(slot: Box<ServerSlot>) -> BattleScheduler {
        BattleScheduler{slot: slot, waiting: RingBuf::new()}
    }

    pub fn run(&mut self) {
        loop {
            match self.slot.receive() {
                Joined(client_id) => {
                    println!("Client {} joined the scheduler", client_id);
                    self.waiting.push(client_id);
                    self.try_schedule();
                },
                ReceivedPacket(client_id, packet) => {
                    println!("Received packet from {} of length {}", client_id, packet.len());
                },
                _ => {}
            }
        }
    }
    
    pub fn try_schedule(&mut self) {
        if self.waiting.len() >= 2 {
            let new_slot = self.slot.create_slot();
            self.slot.transfer_client(self.waiting.pop().unwrap(), new_slot.id());
            self.slot.transfer_client(self.waiting.pop().unwrap(), new_slot.id());
            spawn(proc() {
                let mut battle_state = ServerBattleState::new(new_slot);
                battle_state.run();
            });
        }
    }
}