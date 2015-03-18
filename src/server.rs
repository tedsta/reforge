#![crate_name = "reforge_server"]
#![crate_type = "bin"]
#![feature(box_syntax)]

extern crate bincode;
extern crate time;
extern crate "rustc-serialize" as rustc_serialize;

use std::thread::Thread;
use std::sync::mpsc::channel;

use net::Server;
use star_map_server::StarMapServer;

mod ai;
mod assets;
mod battle_state;
mod battle_type;
mod login;
mod module;
mod net;
mod sector_data;
mod sector_state;
mod ship;
mod sim;
mod sim_events;
mod vec;

mod battle_scheduler;
mod server_battle_state;
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
