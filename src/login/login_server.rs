use std::sync::mpsc::{Sender, Receiver};

use net::{
    OutPacket,
    ServerSlot,
    ServerSlotId,
    SlotInMsg,
};

use super::{
    AccountBox,
    AccountManager,
    LoginError,
};
use super::LoginPacket;
use ship::{Ship, ShipId, ShipStored};

pub fn run_login_server(slot: ServerSlot,
                        star_map_slot_id: ServerSlotId,
                        star_map_chan: Sender<AccountBox>,
                        logout_receiver: Receiver<AccountBox>) {
    let mut account_manager = AccountManager::new();

    loop {
        if let Ok(msg) = slot.try_receive() {
            match msg {
                SlotInMsg::Joined(client_id) => {
                    println!("Client {} logging in...", client_id);
                },
                SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                    let LoginPacket{username: username, password: password} = packet.read().ok().expect("Failed to receive login packet");
                    
                    match account_manager.login_account(username.clone(), password.clone(), client_id) {
                        Ok(account) => {
                            // Login ok
                            let mut result_packet = OutPacket::new();
                            let login_result: Option<LoginError> = None;
                            result_packet.write(&login_result);
                            slot.send(client_id, result_packet);
                            
                            slot.transfer_client(account.client_id.expect("This must have a client ID"), star_map_slot_id);
                            star_map_chan.send(account);
                        },
                        Err(ref e) if *e == LoginError::NoSuchAccount => {
                            // Login ok
                            let mut result_packet = OutPacket::new();
                            let login_result: Option<LoginError> = None;
                            result_packet.write(&login_result);
                            slot.send(client_id, result_packet);
                        
                            // Account doesn't exist yet, make it
                            account_manager.create_account(username.clone(), password.clone());
                            
                            // Log into the new account
                            if let Ok(mut account) = account_manager.login_account(username.clone(), password.clone(), client_id) {
                                // Create ships
                                let player_ship = ShipStored::from_ship(Ship::generate(client_id as ShipId, username.clone(), 5));
                                
                                account.ship = Some(player_ship);
                                
                                slot.transfer_client(account.client_id.expect("This must have a client ID"), star_map_slot_id);
                                star_map_chan.send(account);
                                
                            } else {
                                panic!("Failed to log into newly created account");
                            }
                        },
                        Err(e) => {
                            let mut result_packet = OutPacket::new();
                            result_packet.write(&Some(e));
                            slot.send(client_id, result_packet);
                        },
                    }
                },
                _ => {},
            }
        }
        
        if let Ok(account) = logout_receiver.try_recv() {
            println!("Client {} logging out", account.client_id.expect("This must have a client ID"));
            account_manager.logout_account(account);
        }        
    }
}