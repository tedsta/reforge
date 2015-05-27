#![crate_name = "reforge_server"]
#![crate_type = "bin"]
#![feature(box_syntax)]
#![feature(rand)]
#![feature(core)]
#![feature(os)]
#![feature(io)]
#![feature(old_io)]
#![feature(alloc)]
#![feature(thread_sleep)]
#![feature(collections)]
#![feature(std_misc)]
#![feature(convert)]
#![feature(collections_drain)]
#![feature(duration)]

extern crate bincode;
extern crate float;
extern crate num;
extern crate rand;
extern crate rustc_serialize;
extern crate time;

use std::thread::Builder;
use std::sync::mpsc::channel;

use net::Server;
use star_map::StarMapServer;

mod ai;
mod battle_context;
mod battle_type;
mod chat;
mod client_action;
mod login;
mod module;
mod net;
mod no_encode;
mod packet_types;
mod sector_data;
mod sector_server;
mod ship;
mod sim;
mod sim_events;
mod star_map;
mod vec;

fn main() {
    let mut server = Server::new();
    let login_slot = server.create_slot();
    let star_map_slot = server.create_slot();
    let star_map_slot_id = star_map_slot.get_id();
    let (star_map_account_sender, star_map_account_receiver) = channel();
    let (logout_sender, logout_receiver) = channel();
    
    Builder::new().name("server_master".to_string()).spawn(move || {
        server.listen("0.0.0.0:30000");
    });
    
    Builder::new().name("login_server".to_string()).spawn(move || {
        login::run_login_server(login_slot, star_map_slot_id, star_map_account_sender, logout_receiver);
    });
    
    let mut star_map_server = StarMapServer::new(star_map_slot);
    star_map_server.run(star_map_account_receiver, logout_sender);
}
