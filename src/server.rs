#![crate_name = "spacegame_server"]
#![desc = "spacegame awesome mmo server"]
#![crate_type = "bin"]

use net::{Server, Joined, ReceivedPacket};

pub mod net;

fn main() {
    let mut server = Server::new();
    let slot = server.create_slot();
    
    spawn(proc() {
            server.listen(30000);
        });

    loop {
        match slot.receive() {
            ReceivedPacket(_, mut packet) => {
                println!("Server got: {}, {}, {}", packet.read_int().unwrap(), packet.read_uint().unwrap(), packet.read_int().unwrap());
            },
            Joined(client_id) => println!("{} has joined.", client_id),
            _ => {}
        }
    }
}