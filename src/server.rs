#![crate_name = "reforge_server"]
#![crate_type = "bin"]
#![feature(globs)]
#![feature(macro_rules)]

extern crate bincode;
extern crate "rustc-serialize" as serialize;

use net::Server;
use battle_scheduler::BattleScheduler;

pub mod ai;
pub mod assets;
pub mod battle_scheduler;
pub mod battle_state;
pub mod battle_type;
pub mod module;
pub mod net;
pub mod server_battle_state;
pub mod ship;
pub mod sim;
pub mod vec;

fn main() {
    let mut server = Server::new();
    let slot = server.create_slot();
    
    spawn(move || {
        server.listen("0.0.0.0:30000");
    });
        
    let mut scheduler = BattleScheduler::new(slot);
    scheduler.run();
}
