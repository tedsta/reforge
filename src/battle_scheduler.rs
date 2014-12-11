use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{RingBuf, HashMap};
use std::sync::Arc;

use battle_state::BattleContext;
use module::ModuleTypeStore;
use net::{ClientId, ServerSlot, Joined, ReceivedPacket};
use server_battle_state::ServerBattleState;
use ship::{Ship, ShipId};

pub struct BattleScheduler {
    slot: Box<ServerSlot>,
    waiting: RingBuf<ClientId>, // Clients still waiting for a battle
    
    mod_store: Arc<ModuleTypeStore>,
}

impl BattleScheduler {
    pub fn new(slot: Box<ServerSlot>, mod_store: Arc<ModuleTypeStore>) -> BattleScheduler {
        BattleScheduler{slot: slot, waiting: RingBuf::new(), mod_store: mod_store}
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
            self.slot.transfer_client(client1, new_slot.get_id());
            self.slot.transfer_client(client2, new_slot.get_id());
            
            let mod_store = self.mod_store.clone();
            
            spawn(proc() {
                // Create ships
                let mut ship1 = Ship::generate(mod_store.deref(), client1 as ShipId);
                ship1.client_id = Some(client1);
                let mut ship2 = Ship::generate(mod_store.deref(), client2 as ShipId);
                ship2.client_id = Some(client2);
            
                // Map clients to their ships
                let mut ships = HashMap::new();
                ships.insert(client1, Rc::new(RefCell::new(ship1)));
                ships.insert(client2, Rc::new(RefCell::new(ship2)));
            
                let mut battle_state = ServerBattleState::new(new_slot, BattleContext::new(ships));
                battle_state.run();
            });
            
        }
    }
}