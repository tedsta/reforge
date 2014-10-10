use std::collections::{RingBuf, HashMap};

use battle_state::BattleContext;
use net::{ClientId, ServerSlot, Joined, ReceivedPacket};
use server_battle_state::ServerBattleState;
use ship::Ship;

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
            
            let client1 = self.waiting.pop().expect("First client wasn't there somehow");
            let client2 = self.waiting.pop().expect("Second client wasn't there somehow");
            
            // Transfer clients to battle state
            self.slot.transfer_client(client1, new_slot.id());
            self.slot.transfer_client(client2, new_slot.id());
            
            spawn(proc() {
                // Map clients to their ships
                let mut ships = HashMap::new();
                ships.insert(client1, Ship::generate(client1 as u64));
                ships.insert(client2, Ship::generate(client2 as u64));
            
                let mut battle_state = ServerBattleState::new(new_slot, BattleContext::new(ships));
                battle_state.run();
            });
            
        }
    }
}