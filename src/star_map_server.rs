use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::Thread;

use battle_state::BattleContext;
use login::AccountBox;
use net::{
    ServerSlot,
    ServerSlotId,
    SlotInMsg,
};
use sector_state::SectorState;

#[derive(PartialEq, Eq, Hash, RustcEncodable, RustcDecodable)]
pub struct SectorId(pub u32);

pub struct StarMapServer {
    slot: ServerSlot,
    sectors: HashMap<SectorId, (ServerSlotId, Sender<AccountBox>, Receiver<AccountBox>)>,
}

impl StarMapServer {
    pub fn new(slot: ServerSlot) -> StarMapServer {
        let mut sectors = HashMap::new();
        
        let (to_sector_sender, to_sector_receiver) = channel();
        let (from_sector_sender, from_sector_receiver) = channel();
        let sector_slot = slot.create_slot();
        sectors.insert(SectorId(0), (sector_slot.get_id(), to_sector_sender, from_sector_receiver));
        
        Thread::spawn(move || {
            let mut sector_state = SectorState::new(sector_slot, BattleContext::new(vec!()));
            sector_state.run(from_sector_sender, to_sector_receiver);
        });
        
        StarMapServer {
            slot: slot,
            sectors: sectors,
        }
    }
    
    pub fn run(&mut self, account_receiver: Receiver<AccountBox>) {
        loop {
            if let Ok(slot_msg) = self.slot.try_receive() {
                match slot_msg {
                    SlotInMsg::Joined(client_id) => {
                        println!("Client {} joined the star map", client_id);
                    },
                    SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                    },
                    _ => {},
                }
            }
            
            if let Ok(account) = account_receiver.try_recv() {
                let (ref sector_slot_id, ref to_sector_account_sender, _) = self.sectors[account.sector];
                
                self.slot.transfer_client(account.client_id.expect("This needs to have a client ID"), *sector_slot_id);
                to_sector_account_sender.send(account);
            }
        }
    }
}