#![crate_name = "spacegame"]
#![desc = "spacegame awesome mmo"]
#![crate_type = "bin"]

extern crate native;
extern crate rsfml;

use rsfml::graphics::{RenderWindow, Color};
use rsfml::window::{VideoMode, ContextSettings, keyboard, Close};

use input_system::InputSystem;
use input_system::KeyHandler;

use net::{Server, Client, Joined, ReceivedPacket, OutPacket};

pub mod input_system;
pub mod net;

struct Foo;

impl KeyHandler for Foo {
    fn on_key_pressed(&mut self, key: keyboard::Key) {
        println!("Pressed {}!", key);
    }
    
    fn on_key_released(&mut self, key: keyboard::Key) {
        println!("Released {}!", key);
    }
}

#[cfg(target_os="macos")]
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}


fn main () -> () {
    // https://github.com/jeremyletang/rust-sfml/issues/37
    /*unsafe { ::std::rt::stack::record_sp_limit(0); }
    
    let mut input_sys = InputSystem::new();
    
    let mut foo = Foo;
    input_sys.add_key_handler(&mut foo);

    // Create the window of the application
    let setting: ContextSettings = ContextSettings::default();
    let mut window: RenderWindow =
        match RenderWindow::new(VideoMode::new_init(800, 600, 32),
                                "spacegame",
                                Close,
                                &setting) {
            Some(window) => window,
            None => fail!("Cannot create a new Render Window.")
        };

    while window.is_open() {
        // Poll for inputs
        input_sys.update(&mut window);
        
        // Clear the screen
        window.clear(&Color::black());

        // Display things on screen
        window.display();
    }*/
    
    let mut server = Server::new();
    let slot = server.create_slot();
    
    spawn(proc() {
            server.listen(30000);
        });
        
    let mut client = Client::new("127.0.0.1", 30000);
    let mut client2 = Client::new("127.0.0.1", 30000);
    
    let mut packet = OutPacket::new();
    packet.write_int(42).unwrap();
    packet.write_uint(444422).unwrap();
    packet.write_int(64).unwrap();

    client.send(&packet);
    client2.send(&packet);
    
    slot.send(1, packet);
    //slot.send(1, packet);
    
    spawn(proc() {
        loop {
            match slot.receive() {
                ReceivedPacket(_, mut packet) => {
                    println!("Server got: {}, {}, {}", packet.read_int().unwrap(), packet.read_uint().unwrap(), packet.read_int().unwrap());
                },
                Joined(client_id) => println!("{} has joined.", client_id),
                _ => {}
            }
        }
    });
    
    spawn(proc() {
        let mut packet = client.receive();
        println!("client got: {}, {}, {}", packet.read_int().unwrap(), packet.read_uint().unwrap(), packet.read_int().unwrap());
    });
    
    spawn(proc() {
        let mut packet = client2.receive();
        println!("client2 got: {}, {}, {}", packet.read_int().unwrap(), packet.read_uint().unwrap(), packet.read_int().unwrap());
    });
}