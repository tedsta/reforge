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

extern crate bincode;
extern crate time;
extern crate rustc_serialize;

use std::thread::Thread;
use std::sync::mpsc::channel;

use net::Server;
use star_map_server::StarMapServer;

mod ai;
mod battle_context;
mod battle_type;
mod client_action;
mod login;
mod module;
mod net;
mod sector_data;
mod sector_server;
mod ship;
mod sim;
mod sim_events;
mod station_server;
mod vec;

mod star_map_server;

fn main() {
    // Start a local server
    let mut server = Server::new();
    let login_slot = server.create_slot();
    let star_map_slot = server.create_slot();
    let star_map_slot_id = star_map_slot.get_id();
    let (star_map_account_sender, star_map_account_receiver) = channel();
    
    Thread::spawn(move || {
        server.listen("0.0.0.0:30000");
    });
    
    Thread::spawn(move || {
        login::run_login_server(login_slot, star_map_slot_id, star_map_account_sender);
    });
    
    let mut star_map_server = StarMapServer::new(star_map_slot);
    star_map_server.run(star_map_account_receiver);
}
