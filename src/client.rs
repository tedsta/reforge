#![crate_name = "reforge_client"]
#![crate_type = "bin"]
#![feature(macro_rules)]
#![feature(default_type_params)]
#![feature(globs)]
#![feature(box_syntax)]

extern crate bincode;
extern crate time;
extern crate "rustc-serialize" as rustc_serialize;

// Piston stuff
extern crate quack;
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
use std::thread::Thread;

use sdl2_window::Sdl2Window;
use opengl_graphics::Gl;

use asset_store::AssetStore;
use battle_state::BattleContext;
use battle_type::BattleType;
use client_battle_state::ClientBattleState;
use main_menu::{MainMenu, MainMenuSelection};
use net::{Client, OutPacket};

// Server stuff
use net::Server;
use battle_scheduler::BattleScheduler;

#[macro_escape]
pub mod util;

pub mod assets;
pub mod asset_store;
pub mod battle_state;
pub mod battle_type;
pub mod client_battle_state;
pub mod main_menu;
pub mod module;
pub mod net;
pub mod ship;
pub mod sim;
pub mod space_gui;
pub mod sprite_sheet;
pub mod vec;

// server stuff
pub mod ai;
pub mod battle_scheduler;
pub mod server_battle_state;

fn main () {
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

    // Wrap window in RefCell
    let window = RefCell::new(window);
    
    // Create main menu
    let mut main_menu = MainMenu::new();
    let selection = main_menu.run(&window, &mut gl, &asset_store);

    match selection {
        MainMenuSelection::SinglePlayer => {
            // Start a local server
            let mut server = Server::new();
            let slot = server.create_slot();
            
            Thread::spawn(move || {
                server.listen("localhost:30000");
            });
            
            Thread::spawn(move || {
                let mut scheduler = BattleScheduler::new(slot);
                scheduler.run();
            });
        
            // Connect to server
            let mut client = Client::new("localhost:30000");

            let mut packet = OutPacket::new();
            packet.write(&BattleType::Ai);
            client.send(&packet);
            
            // Receive the ships from the server
            let mut packet = client.receive();
            let ships = match packet.read() {
                Ok(ships) => ships,
                Err(e) => panic!("Unable to receive ships froms server: {}", e),
            };
            
            // Create the battle state
            let mut battle = ClientBattleState::new(client, BattleContext::new(ships));

            battle.run(&window, &mut gl, &asset_store);
        },
        MainMenuSelection::Multiplayer => {
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
            
            // Connect to server
            let mut client = Client::new(ip_address.as_slice());

            let mut packet = OutPacket::new();
            packet.write(&BattleType::FreeForAll{num_players: 2});
            client.send(&packet);
            
            // Receive the ships from the server
            let mut packet = client.receive();
            let ships = match packet.read() {
                Ok(ships) => ships,
                Err(e) => panic!("Unable to receive ships froms server: {}", e),
            };
            
            // Create the battle state
            let mut battle = ClientBattleState::new(client, BattleContext::new(ships));

            battle.run(&window, &mut gl, &asset_store);
        },
        MainMenuSelection::Exit => {
            
        },
    }
}
