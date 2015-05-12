use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::Builder;
use time;

use battle_context::BattleContext;
use chat::ChatServer;
use client_action::ClientAction;
use login::AccountBox;
use module::ModelStore;
use net::{
    OutPacket,
    ServerSlot,
    ServerSlotId,
    SlotInMsg,
};
use sector_data::{SectorData, SectorId, SectorKind};
use sector_server::SectorState;
use super::station::StationServer;
use vec::Vec2;

// Reason a ship is leaving a sector
pub enum StarMapAction {
    Jump(SectorId),
    Logout,
}

pub struct Sector {
    pub slot_id: ServerSlotId,
    pub to_sector: Sender<AccountBox>,
    pub from_sector: Receiver<(AccountBox, StarMapAction)>,
    pub ack: Receiver<()>,
    pub data: SectorData,
}

pub struct StarMapServer {
    slot: ServerSlot,
    sectors: HashMap<SectorId, Sector>,
    
    jumping_accounts: VecDeque<(AccountBox, SectorId, time::Timespec)>,
}

impl StarMapServer {
    pub fn new(slot: ServerSlot) -> StarMapServer {
        // Chat server input channel
        let (to_chat_server, chat_from_sector) = channel();
        let mut chat_msg_senders = vec!();
        
        // Fire up the universe
    
        let model_store = Arc::new(ModelStore::new());
        let slot_id = slot.get_id();
        let mut sectors = HashMap::new();
        
        // Station
        let (to_sector_sender, to_sector_receiver) = channel();
        let (from_sector_sender, from_sector_receiver) = channel();
        let (ack_sender, ack_receiver) = channel();
        let (chat_sender, sector_chat_in) = channel();
        chat_msg_senders.push(chat_sender);
        let sector_slot = slot.create_slot();
        let sector_id = SectorId(0);
        let sector_chat_out = to_chat_server.clone();
        sectors.insert(sector_id, Sector {
            slot_id: sector_slot.get_id(),
            to_sector: to_sector_sender,
            from_sector: from_sector_receiver,
            ack: ack_receiver,
            data: SectorData {
                id: sector_id,
                kind: SectorKind::Station,
                map_position: Vec2 { x: 100.0, y: 75.0 },
            },
        });
        
        
        Builder::new()
            .name(format!("station_{}_thread", 0))
            .spawn(move || {
                let mut sector_server = StationServer::new(sector_slot,
                                                           slot_id,
                                                           sector_chat_out,
                                                           sector_chat_in,
                                                           model_store.clone());
                sector_server.run(from_sector_sender, to_sector_receiver, ack_sender);
            });
        
        // Sector 1
        let (to_sector_sender, to_sector_receiver) = channel();
        let (from_sector_sender, from_sector_receiver) = channel();
        let (ack_sender, ack_receiver) = channel();
        let (chat_sender, sector_chat_in) = channel();
        chat_msg_senders.push(chat_sender);
        let sector_slot = slot.create_slot();
        let sector_id = SectorId(1);
        let sector_chat_out = to_chat_server.clone();
        sectors.insert(sector_id, Sector {
            slot_id: sector_slot.get_id(),
            to_sector: to_sector_sender,
            from_sector: from_sector_receiver,
            ack: ack_receiver,
            data: SectorData {
                id: sector_id,
                kind: SectorKind::Sector,
                map_position: Vec2 { x: 50.0, y: 50.0 },
            },
        });
        
        Builder::new()
            .name(format!("sector_{}_thread", 1))
            .spawn(move || {
                let mut sector_server = SectorState::new(sector_slot,
                                                         slot_id,
                                                         sector_chat_out,
                                                         sector_chat_in,
                                                         BattleContext::new(vec!()),
                                                         false);
                sector_server.run(from_sector_sender, to_sector_receiver, ack_sender, false);
            });
        
        // Sector 2
        let (to_sector_sender, to_sector_receiver) = channel();
        let (from_sector_sender, from_sector_receiver) = channel();
        let (ack_sender, ack_receiver) = channel();
        let (chat_sender, sector_chat_in) = channel();
        chat_msg_senders.push(chat_sender);
        let sector_slot = slot.create_slot();
        let sector_id = SectorId(2);
        let sector_chat_out = to_chat_server.clone();
        sectors.insert(sector_id, Sector {
            slot_id: sector_slot.get_id(),
            to_sector: to_sector_sender,
            from_sector: from_sector_receiver,
            ack: ack_receiver,
            data: SectorData {
                id: sector_id,
                kind: SectorKind::Sector,
                map_position: Vec2 { x: 100.0, y: 100.0 },
            },
        });
        
        Builder::new()
            .name(format!("sector_{}_thread", 2))
            .spawn(move || {
                let mut sector_server = SectorState::new(sector_slot,
                                                         slot_id,
                                                         sector_chat_out,
                                                         sector_chat_in,
                                                         BattleContext::new(vec!()),
                                                         false);
                sector_server.run(from_sector_sender, to_sector_receiver, ack_sender, true);
            });
        
        // Start the chat server
        Builder::new()
            .name("chat_server".to_string())
            .spawn(move || {
                let mut chat_server = ChatServer::new(chat_from_sector, chat_msg_senders);
                chat_server.run();
            });
        
        ////////////////////////////////////////////////////////////////////////////////////////////
        
        StarMapServer {
            slot: slot,
            sectors: sectors,
            jumping_accounts: VecDeque::new(),
        }
    }
    
    pub fn run(&mut self, from_login: Receiver<AccountBox>) {
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
            
            if let Ok(account) = from_login.try_recv() {
                let client_id = account.client_id.expect("This needs to have a client ID");
            
                let sector_data: Vec<SectorData> = self.sectors.iter().map(|(_, s)| s.data.clone()).collect();
                
                let ref sector = self.sectors[&account.sector];
            
                let mut sectors_packet = OutPacket::new();
                sectors_packet.write(&sector_data).unwrap();
                self.slot.send(client_id, sectors_packet);
                
                ////////////////////////////////////////////////////////////////////////////////////
                
                let client_action =
                    match sector.data.kind {
                        SectorKind::Sector => ClientAction::JoinSector,
                        SectorKind::Station => ClientAction::JoinStation,
                    };
                
                let mut action_packet = OutPacket::new();
                action_packet.write(&client_action).unwrap();
                self.slot.send(client_id, action_packet);
                
                sector.to_sector.send(account);
                sector.ack.recv();
                self.slot.transfer_client(client_id, sector.slot_id);
            }
            
            // Send any jumping ships to their new sector
            for sector in self.sectors.values() {
                if let Ok((account, exit_action)) = sector.from_sector.try_recv() {
                    match exit_action {
                        StarMapAction::Jump(sector) => {
                            self.jumping_accounts.push_back((account, sector, time::now().to_timespec() + time::Duration::milliseconds(6000)));
                        },
                        _ => { },
                    }
                }
            }
            
            while let Some((mut account, target_sector, jump_time)) = self.jumping_accounts.pop_front() {
                if (time::now().to_timespec() - jump_time).num_milliseconds() < 0 {
                    self.jumping_accounts.push_front((account, target_sector, jump_time));
                    break;
                } else {
                    let client_id = account.client_id.expect("This needs to have a client ID");
                    
                    let ref sector = self.sectors[&target_sector];
                    
                    let client_action =
                        match sector.data.kind {
                            SectorKind::Sector => ClientAction::JoinSector,
                            SectorKind::Station => ClientAction::JoinStation,
                        };
                
                    let mut action_packet = OutPacket::new();
                    action_packet.write(&client_action).unwrap();
                    self.slot.send(client_id, action_packet);
                    
                    sector.to_sector.send(account);
                    sector.ack.recv();
                    self.slot.transfer_client(client_id, sector.slot_id);
                }
            }
        }
    }
}