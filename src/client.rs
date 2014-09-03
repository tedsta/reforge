#![crate_name = "spacegame_client"]
#![desc = "spacegame awesome mmo client"]
#![crate_type = "bin"]

extern crate native;
extern crate rsfml;

use rsfml::graphics::{RenderWindow, Color};
use rsfml::window::{VideoMode, ContextSettings, keyboard, Close};

use client_battle_state::ClientBattleState;
use input::InputSystem;
use input::KeyHandler;
use net::{Client, OutPacket};

pub mod battle_state_packets;
pub mod client_battle_state;
pub mod input;
pub mod module;
pub mod net;
pub mod ship;
pub mod sim_element;

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


fn main () {
    // https://github.com/jeremyletang/rust-sfml/issues/37
    unsafe { ::std::rt::stack::record_sp_limit(0); }
    
    let mut input_sys = InputSystem::new();
    
    input_sys.add_key_handler(box Foo);

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
    
    // Connect to server
    let mut client = Client::new("127.0.0.1", 30000);
    
    // Create the battle state
    let mut battle = ClientBattleState::new(client);

    battle.run(&mut window, &mut input_sys);
}