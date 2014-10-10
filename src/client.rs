#![crate_name = "spacegame_client"]
#![desc = "spacegame awesome mmo client"]
#![crate_type = "bin"]

extern crate binary_encode;
extern crate native;
extern crate time;
extern crate serialize;
extern crate rsfml;

use rsfml::graphics::{RenderWindow};
use rsfml::window::{VideoMode, ContextSettings, keyboard, Close};

use client_battle_state::ClientBattleState;
use input::InputSystem;
use input::KeyHandler;
use net::Client;
use sfml_renderer::SfmlRenderer;

pub mod battle_state;
pub mod client_battle_state;
pub mod input;
pub mod module;
pub mod net;
pub mod render;
pub mod sfml_renderer;
pub mod ship;
pub mod sim_element;
pub mod space_gui;
pub mod vec;

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
    let window: RenderWindow =
        match RenderWindow::new(VideoMode::new_init(1280, 768, 32),
                                "spacegame",
                                Close,
                                &setting) {
            Some(window) => window,
            None => fail!("Cannot create a new Render Window.")
        };
    
    let mut renderer = SfmlRenderer::new(window);
    
    // Connect to server
    let mut client = Client::new("127.0.0.1", 30000);
    
    // Receive the battle context from the server
    let mut packet = client.receive();
    let context = match packet.read() {
        Ok(context) => context,
        Err(e) => fail!("Unable to receive battle context froms server: {}", e),
    };
    
    // Create the battle state
    let mut battle = ClientBattleState::new(client, context);

    battle.run(&mut renderer, &mut input_sys);
}