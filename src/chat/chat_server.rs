use std::sync::mpsc::{Sender, Receiver};

use super::ChatMsg;

pub struct ChatServer {
    msg_receiver: Receiver<ChatMsg>,
    msg_senders: Vec<Sender<ChatMsg>>,
    msg_log: Vec<ChatMsg>,
}

impl ChatServer {
    pub fn new(msg_receiver: Receiver<ChatMsg>, msg_senders: Vec<Sender<ChatMsg>>) -> ChatServer {
        ChatServer {
            msg_receiver: msg_receiver,
            msg_senders: msg_senders,
            msg_log: vec!(),
        }
    }
    
    pub fn run(&mut self) {
        while let Ok(msg) = self.msg_receiver.recv() {
            for msg_sender in &self.msg_senders {
                msg_sender.send(msg.clone());
            }
            self.add_msg(msg);
        }
        
        println!("Chat log died");
    }
    
    pub fn add_msg(&mut self, msg: ChatMsg) {
        self.msg_log.push(msg);
    }
}