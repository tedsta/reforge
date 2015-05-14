use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};

use chat::ChatMsg;
use login::AccountBox;
use module::ModelStore;
use net::{ClientId, ServerSlot, ServerSlotId, SlotInMsg, InPacket, OutPacket};
use star_map::StarMapAction;
use star_map::station::{ShipEditAction, StationAction};

pub struct StationServer {
    slot: ServerSlot,
    star_map_slot_id: ServerSlotId,
    chat_sender: Sender<ChatMsg>,
    chat_receiver: Receiver<ChatMsg>,
    
    model_store: Arc<ModelStore>,

    // All the clients' accounts
    accounts: HashMap<ClientId, AccountBox>,
}

impl StationServer {
    pub fn new(slot: ServerSlot,
               star_map_slot_id: ServerSlotId,
               chat_sender: Sender<ChatMsg>,
               chat_receiver: Receiver<ChatMsg>,
               model_store: Arc<ModelStore>) -> StationServer {
        StationServer {
            slot: slot,
            star_map_slot_id: star_map_slot_id,
            chat_sender: chat_sender,
            chat_receiver: chat_receiver,
            model_store: model_store,
            accounts: HashMap::new(),
        }
    }
    
    pub fn run(&mut self,
               to_map_sender: Sender<(AccountBox, StarMapAction)>,
               from_map_receiver: Receiver<AccountBox>,
               ack: Sender<()>) {    
        loop {
            ///////////////////////////////////////////////////////////
            // Receiver ServerSlot messages
            if let Ok(msg) = self.slot.try_receive() {
                match msg {
                    SlotInMsg::Joined(client_id) => {
                        println!("Client {} joined station {}", client_id, self.slot.get_id());
                    },
                    SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                        self.handle_packet(client_id, &mut packet, &to_map_sender);
                    },
                    _ => {}
                }
            }
            
            ///////////////////////////////////////////////////////////
            // Receive messages from chat server
            if let Ok(msg) = self.chat_receiver.try_recv() {
                let mut msg_packet = OutPacket::new();
                msg_packet.write(&msg).unwrap();
                self.slot.broadcast(msg_packet);
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
    
    fn handle_packet(&mut self, client_id: ClientId, packet: &mut InPacket, to_map_sender: &Sender<(AccountBox, StarMapAction)>) {
        let action: StationAction = packet.read().ok().expect("Failed to read StationAction packet");

        match action {
            StationAction::Jump(sector) => {
                let mut account = self.accounts.remove(&client_id).expect("Client's account must exist here.");
                
                self.slot.transfer_client(client_id, self.star_map_slot_id);
                
                to_map_sender.send((account, StarMapAction::Jump(sector)));
            },
            StationAction::ShipEdit(ship_edit) => {
                let ref mut account = self.accounts.get_mut(&client_id).expect("Client's account must exist here.");
                match account.ship {
                    Some(ref mut ship) => {
                        match ship_edit {
                            ShipEditAction::Place(model, x, y) => {
                                let mut module = model.get(&*self.model_store).create();
                                module.x = x;
                                module.y = y;
                                
                                ship.add_module(module);
                            },
                            ShipEditAction::Remove(module) => {
                            },
                        }
                    },
                    None => {
                        println!("Player without ship tried to edit ship");
                    },
                }
            },
            StationAction::Chat(msg) => {
                let ref account = self.accounts[&client_id];
            
                let msg = ChatMsg {
                    author_name: account.username.clone(),
                    content: msg,
                };
                
                self.chat_sender.send(msg);
            },
            StationAction::Logout => {
                let mut account = self.accounts.remove(&client_id).expect("Client's account must exist here.");
                
                self.slot.transfer_client(client_id, self.star_map_slot_id);
                
                to_map_sender.send((account, StarMapAction::Logout));
            },
        }
    }
}
