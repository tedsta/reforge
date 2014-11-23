#![crate_name = "reforge_server"]
#![desc = "reforge awesome mmo server"]
#![crate_type = "bin"]

extern crate bincode;
extern crate serialize;

use std::sync::Arc;

use net::Server;
use battle_scheduler::BattleScheduler;
use module::ModuleTypeStore;

pub mod assets;
pub mod battle_scheduler;
pub mod battle_state;
pub mod module;
pub mod net;
pub mod server_battle_state;
pub mod ship;
pub mod sim;
pub mod vec;

fn main() {
    let mut server = Server::new();
    let slot = box server.create_slot();
    
    spawn(proc() {
        server.listen("0.0.0.0:30000");
    });

    let mod_store = Arc::new(ModuleTypeStore::new());
        
    let mut scheduler = BattleScheduler::new(slot, mod_store);
    scheduler.run();
}