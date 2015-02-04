use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use login::AccountBox;
use net::{
    ServerSlot,
    SlotInMsg,
};

#[derive(PartialEq, Eq, Hash, RustcEncodable, RustcDecodable)]
pub struct SectorId(u32);

pub struct StarMapServer {
    sectors: HashMap<SectorId, Receiver<()>>,
}

impl StarMapServer {
    pub fn new() -> StarMapServer {
        StarMapServer {
            sectors: HashMap::new(),
        }
    }
    
    pub fn run(&mut self, slot: ServerSlot, account_receiver: Receiver<AccountBox>) {
        loop {
            match slot.receive() {
                SlotInMsg::Joined(client_id) => {
                    println!("Client {} joined the star map", client_id);
                },
                SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                },
                _ => {},
            }
        }
    }
}