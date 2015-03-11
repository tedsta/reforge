use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::Builder;

use battle_state::BattleContext;
use login::AccountBox;
use net::{
    OutPacket,
    ServerSlot,
    ServerSlotId,
    SlotInMsg,
};
use sector_data::{SectorData, SectorId};
use sector_state::SectorState;
use vec::Vec2;

pub struct Sector {
    pub slot_id: ServerSlotId,
    pub to_sector_sender: Sender<AccountBox>,
    pub from_sector_receiver: Receiver<AccountBox>,
    pub data: SectorData,
}

pub struct StarMapServer {
    slot: ServerSlot,
    sectors: HashMap<SectorId, Sector>,
}

impl StarMapServer {
    pub fn new(slot: ServerSlot) -> StarMapServer {
        let mut sectors = HashMap::new();
        
        // Sector 0
        let (to_sector_sender, to_sector_receiver) = channel();
        let (from_sector_sender, from_sector_receiver) = channel();
        let sector_slot = slot.create_slot();
        let sector_id = SectorId(0);
        sectors.insert(sector_id, Sector {
            slot_id: sector_slot.get_id(),
            to_sector_sender: to_sector_sender,
            from_sector_receiver: from_sector_receiver,
            data: SectorData {
                id: sector_id,
                map_position: Vec2 { x: 50.0, y: 50.0 },
            },
        });
        
        Builder::new().name(format!("sector_{}_thread", 0)).spawn(move || {
            let mut sector_state = SectorState::new(sector_slot, BattleContext::new(vec!()));
            sector_state.run(from_sector_sender, to_sector_receiver);
        });
        
        // Sector 1
        let (to_sector_sender, to_sector_receiver) = channel();
        let (from_sector_sender, from_sector_receiver) = channel();
        let sector_slot = slot.create_slot();
        let sector_id = SectorId(1);
        sectors.insert(sector_id, Sector {
            slot_id: sector_slot.get_id(),
            to_sector_sender: to_sector_sender,
            from_sector_receiver: from_sector_receiver,
            data: SectorData {
                id: sector_id,
                map_position: Vec2 { x: 100.0, y: 100.0 },
            },
        });
        
        Builder::new().name(format!("sector_{}_thread", 1)).spawn(move || {
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
                let client_id = account.client_id.expect("This needs to have a client ID");
            
                let sector_data: Vec<SectorData> = self.sectors.iter().map(|(_, s)| s.data.clone()).collect();
            
                let mut sectors_packet = OutPacket::new();
                sectors_packet.write(&sector_data).ok().expect("Failed to write SectorData");
                self.slot.send(client_id, sectors_packet);
            
                let ref sector = self.sectors[account.sector];
                
                self.slot.transfer_client(client_id, sector.slot_id);
                sector.to_sector_sender.send(account);
            }
        }
    }
}