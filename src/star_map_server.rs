use net::{
    ServerSlot,
    SlotInMsg,
};

pub struct StarMapServer;

impl StarMapServer {
    pub fn new() -> StarMapServer {
        StarMapServer
    }
    
    pub fn run(&mut self, slot: ServerSlot) {
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