use net::{
    ServerSlot,
    SlotInMsg,
};

use super::login_packet::LoginPacket;

pub fn run_login_server(slot: ServerSlot) {
    loop {
        match slot.receive() {
            SlotInMsg::Joined(client_id) => {
                println!("Client {} logging in...", client_id);
            },
            SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                let login_packet: LoginPacket = packet.read().ok().expect("Failed to receive login packet");
            },
            _ => {},
        }
    }
}