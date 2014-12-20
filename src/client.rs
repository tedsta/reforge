#![crate_name = "reforge_client"]
#![desc = "reforge awesome mmo client"]
#![crate_type = "bin"]
#![feature(macro_rules)]
#![feature(default_type_params)]
#![feature(globs)]

extern crate bincode;
extern crate time;
extern crate serialize;

// Piston stuff
extern crate current;
extern crate event;
extern crate graphics;
extern crate input;
extern crate piston;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate shader_version;

use std::io;
use std::os;
use std::cell::RefCell;

use sdl2_window::Sdl2Window;
use opengl_graphics::Gl;

use asset_store::AssetStore;
use battle_state::BattleContext;
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
pub mod ship;
pub mod sim;
pub mod space_gui;
pub mod sprite_sheet;
pub mod vec;

fn main () {
    // Check for IP address in args
    let mut ip_address =
        if os::args().len() > 1 {
            os::args()[1].clone()
        } else {
            print!("IP Address: ");
            let mut ip_address = io::stdin().read_line().unwrap();
            ip_address.pop().unwrap(); // Remove newline at end
            ip_address.pop().unwrap(); // Remove newline at end
            ip_address
        };
    ip_address.push_str(":30000"); // Add the port to the end of the address
    
    let opengl = shader_version::OpenGL::_3_0;
    
    // Create an SDL window.
    let window = Sdl2Window::new(
        opengl,
        piston::WindowSettings {
            title: "reForge".to_string(),
            size: [1280, 720],
            samples: 0,
            fullscreen: false,
            exit_on_esc: true,
        }
    );
    
    // Create GL device
    let mut gl = Gl::new(opengl);
    
    // Create the asset store
    let asset_store = AssetStore::new();
    
    // Connect to server
    let mut client = Client::new(ip_address.as_slice());
    
    // Receive the ships from the server
    let mut packet = client.receive();
    let ships = match packet.read() {
        Ok(ships) => ships,
        Err(e) => panic!("Unable to receive ships froms server: {}", e),
    };
    
    // Wrap window in RefCell
    let window = RefCell::new(window);
    
    // Create the battle state
    let mut battle = ClientBattleState::new(client, BattleContext::new(ships));

    battle.run(&window, &mut gl, &asset_store);
}