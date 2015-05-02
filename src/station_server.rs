use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{Sender, Receiver};

use login::AccountBox;
use module::Module;
use net::{ClientId, ServerSlot, ServerSlotId, SlotInMsg, InPacket, OutPacket};
use ship::{ShipId, ShipStored};
use star_map_server::StarMapAction;

pub struct StationServer {
    slot: ServerSlot,
    star_map_slot_id: ServerSlotId,

    // All the clients' accounts
    accounts: HashMap<ClientId, AccountBox>,
}

impl StationServer {
    pub fn new(slot: ServerSlot, star_map_slot_id: ServerSlotId) -> StationServer {
        StationServer {
            slot: slot,
            star_map_slot_id: star_map_slot_id,
            accounts: HashMap::new(),
        }
    }
    
    pub fn run(
        &mut self,
        to_map_sender: Sender<(AccountBox, StarMapAction)>,
        from_map_receiver: Receiver<AccountBox>,
        ack: Sender<()>,
    ) {    
        loop {
            ///////////////////////////////////////////////////////////
            // Receiver ServerSlot messages
            if let Ok(msg) = self.slot.try_receive() {
                match msg {
                    SlotInMsg::Joined(client_id) => {
                        println!("Client {} joined station {}", client_id, self.slot.get_id());
                    },
                    SlotInMsg::ReceivedPacket(client_id, mut packet) => { self.handle_packet(client_id, &mut packet); },
                    _ => {}
                }
            }
            
            ///////////////////////////////////////////////////////////
            // Receive new clients
            if let Ok(mut account) = from_map_receiver.try_recv() {
                let client_id = account.client_id.expect("This must have a client ID");
                
                // Send initial join packet
                let mut packet = OutPacket::new();
                packet.write(&account.ship).unwrap();
                self.slot.send(client_id, packet);
                
                // Add the player's account
                self.accounts.insert(client_id, account);
                
                ack.send(());
            }
        }
    }
    
    fn handle_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
    }
}
