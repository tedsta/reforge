#![crate_name = "spacegame_server"]
#![desc = "spacegame awesome mmo server"]
#![crate_type = "bin"]

extern crate binary_encode;
extern crate serialize;

use net::Server;
use battle_scheduler::BattleScheduler;

pub mod battle_scheduler;
pub mod battle_state;
pub mod module;
pub mod net;
pub mod render;
pub mod server_battle_state;
pub mod ship;
pub mod sim_element;
pub mod vec;

fn main() {
    let mut server = Server::new();
    let slot = box server.create_slot();
    
    spawn(proc() {
            server.listen(30000);
        });

    let mut scheduler = BattleScheduler::new(slot);
    scheduler.run();
}