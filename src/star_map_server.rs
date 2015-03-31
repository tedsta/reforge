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
    pub to_sector: Sender<AccountBox>,
    pub from_sector: Receiver<AccountBox>,
    pub ack: Receiver<()>,
    pub data: SectorData,
}

pub struct StarMapServer {
    slot: ServerSlot,
    sectors: HashMap<SectorId, Sector>,
}

impl StarMapServer {
    pub fn new(slot: ServerSlot) -> StarMapServer {
        let slot_id = slot.get_id();
    
        let mut sectors = HashMap::new();
        
        // Sector 0
        let (to_sector_sender, to_sector_receiver) = channel();
        let (from_sector_sender, from_sector_receiver) = channel();
        let (ack_sender, ack_receiver) = channel();
        let sector_slot = slot.create_slot();
        let sector_id = SectorId(0);
        sectors.insert(sector_id, Sector {
            slot_id: sector_slot.get_id(),
            to_sector: to_sector_sender,
            from_sector: from_sector_receiver,
            ack: ack_receiver,
            data: SectorData {
                id: sector_id,
                map_position: Vec2 { x: 50.0, y: 50.0 },
            },
        });
        
        Builder::new()
            .name(format!("sector_{}_thread", 0))
            .stack_size(8388608)
            .spawn(move || {
                let mut sector_state = SectorState::new(sector_slot, slot_id, BattleContext::new(vec!()), false);
                sector_state.run(from_sector_sender, to_sector_receiver, ack_sender);
            });
        
        // Sector 1
        let (to_sector_sender, to_sector_receiver) = channel();
        let (from_sector_sender, from_sector_receiver) = channel();
        let (ack_sender, ack_receiver) = channel();
        let sector_slot = slot.create_slot();
        let sector_id = SectorId(1);
        sectors.insert(sector_id, Sector {
            slot_id: sector_slot.get_id(),
            to_sector: to_sector_sender,
            from_sector: from_sector_receiver,
            ack: ack_receiver,
            data: SectorData {
                id: sector_id,
                map_position: Vec2 { x: 100.0, y: 100.0 },
            },
        });
        
        Builder::new()
            .name(format!("sector_{}_thread", 1))
            .stack_size(8388608)
            .spawn(move || {
                let mut sector_state = SectorState::new(sector_slot, slot_id, BattleContext::new(vec!()), false);
                sector_state.run(from_sector_sender, to_sector_receiver, ack_sender);
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
            
                let ref sector = self.sectors[&account.sector];
                
                sector.to_sector.send(account);
                sector.ack.recv();
                self.slot.transfer_client(client_id, sector.slot_id);
            }
            
            // Send any jumping ships to their new sector
            for sector in self.sectors.values() {
                if let Ok(mut account) = sector.from_sector.try_recv() {
                    let client_id = account.client_id.expect("This needs to have a client ID");
                    
                    let target_sector =
                        {
                            let ship = account.ship.as_mut().expect("Ship must exist");
                            ship.target_sector.take().expect("There must be a target sector")
                        };
                    
                    let ref sector = self.sectors[&target_sector];
                    
                    sector.to_sector.send(account);
                    sector.ack.recv();
                    self.slot.transfer_client(client_id, sector.slot_id);
                }
            }
        }
    }
}