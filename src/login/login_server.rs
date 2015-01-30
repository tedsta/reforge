use net::{
    ServerSlot,
    SlotInMsg,
};

use super::account::{
    AccountManager,
    LoginError,
};
use super::login_packet::LoginPacket;

pub fn run_login_server(slot: ServerSlot) {
    let mut account_manager = AccountManager::new();

    loop {
        match slot.receive() {
            SlotInMsg::Joined(client_id) => {
                println!("Client {} logging in...", client_id);
            },
            SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                let LoginPacket{username: username, password: password} = packet.read().ok().expect("Failed to receive login packet");
                
                match account_manager.login_account(username.clone(), password.clone(), client_id) {
                    Ok(account) => {
                        // Login ok
                    },
                    Err(ref e) if *e == LoginError::NoSuchAccount => {
                        // Account doesn't exist yet, make it
                        account_manager.create_account(username, password);
                    },
                    Err(e) => {
                        // TODO tell the client a login error occurred.
                    },
                }
            },
            _ => {},
        }
    }
}