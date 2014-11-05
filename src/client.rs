#![crate_name = "spacegame_client"]
#![desc = "spacegame awesome mmo client"]
#![crate_type = "bin"]
#![feature(macro_rules)]
#![feature(default_type_params)]

extern crate binary_encode;
extern crate native;
extern crate time;
extern crate serialize;
extern crate rsfml;

use rsfml::graphics::{RenderWindow};
use rsfml::window::{VideoMode, ContextSettings, Close};

use asset_store::AssetStore;
use client_battle_state::ClientBattleState;
use net::Client;

#[macro_escape]
pub mod util;

pub mod assets;
pub mod asset_store;
pub mod battle_state;
pub mod client_battle_state;
pub mod module;
pub mod net;
pub mod sfml_renderer;
pub mod ship;
pub mod sim;
pub mod space_gui;
pub mod sprite_sheet;
pub mod vec;

#[cfg(target_os="macos")]
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main () {
    // https://github.com/jeremyletang/rust-sfml/issues/37
    unsafe { ::std::rt::stack::record_sp_limit(0); }
    
    // Create the window of the application
    let setting: ContextSettings = ContextSettings::default();
    let mut window: RenderWindow =
        match RenderWindow::new(VideoMode::new_init(1280, 720, 32),
                                "reForge",
                                Close,
                                &setting) {
            Some(window) => window,
            None => fail!("Cannot create a new Render Window.")
        };
    
    // Create the asset store
    let asset_store = AssetStore::new();
    
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

    battle.run(&mut window, &asset_store);
}