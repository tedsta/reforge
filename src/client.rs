#![crate_name = "reforge_client"]
#![crate_type = "bin"]
#![feature(box_syntax)]

extern crate bincode;
extern crate time;
extern crate "rustc-serialize" as rustc_serialize;

// Piston stuff
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate graphics;
extern crate event;
extern crate input;
extern crate quack;
extern crate shader_version;
extern crate vecmath;
extern crate window;

use std::old_io;
use std::os;
use std::cell::RefCell;
use std::thread::Thread;
use std::sync::mpsc::channel;

use sdl2_window::Sdl2Window;
use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;
use window::WindowSettings;

use asset_store::AssetStore;
use battle_state::BattleContext;
use battle_type::BattleType;
use client_battle_state::ClientBattleState;
use login::LoginPacket;
use main_menu::{MainMenu, MainMenuSelection};
use net::{Client, OutPacket};
use star_map_server::StarMapServer;
use tutorial_state::TutorialState;

// Server stuff
use battle_scheduler::BattleScheduler;
use net::Server;

#[macro_escape]
mod util;

mod ai;
mod assets;
mod asset_store;
mod battle_state;
mod battle_type;
mod client_battle_state;
mod login;
mod main_menu;
mod module;
mod net;
mod sector_state;
mod ship;
mod sim;
mod space_gui;
mod sprite_sheet;
mod tutorial_state;
mod vec;

// server stuff
mod battle_scheduler;
mod server_battle_state;
mod star_map_server;

#[cfg(feature = "client")]
fn main () {
    let opengl = shader_version::OpenGL::_3_2;
    
    // Create an SDL window.
    let window = Sdl2Window::new(
        opengl,
        WindowSettings {
            title: "reForge".to_string(),
            size: [1280, 720],
            samples: 0,
            fullscreen: false,
            exit_on_esc: true,
        }
    );
    
    // Create GL device
    let mut gl = Gl::new(opengl);
    
    // Load our font
    let glyph_cache = GlyphCache::new(&Path::new("content/fonts/8bit.ttf")).unwrap();
    
    // Create the asset store
    let asset_store = AssetStore::new();

    // Wrap window in RefCell
    let window = RefCell::new(window);
    
    // Create main menu
    let mut main_menu = MainMenu::new();
    main_menu.run(&window, &mut gl, |&mut: window, gl, selection| {
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

                battle.run(window, gl, &asset_store, 0);
            },
            MainMenuSelection::Multiplayer => {
                use std::str::StrExt;
                use std::string::String;
            
                // Check for IP address in args
                /*
                let mut ip_address =
                    if os::args().len() > 1 {
                        os::args()[1].clone()
                    } else {
                        print!("IP Address: ");
                        String::from_str(
                            io::stdin().read_line()
                                .ok().expect("Failed to read IP address")
                                .trim_left()
                        )
                    };
                ip_address.push_str(":30000"); // Add the port to the end of the address
                */
                let ip_address = String::from_str("localhost:30000");
                
                // Get credentials
                print!("Username: ");
                let username = String::from_str(
                    old_io::stdin().read_line()
                        .ok().expect("Failed to read username")
                        .trim_left()
                );
                print!("Password: ");
                let password = String::from_str(
                    old_io::stdin().read_line()
                        .ok().expect("Failed to read password")
                        .trim_left()
                );
                
                // Start a local server
                let mut server = Server::new();
                let login_slot = server.create_slot();
                let star_map_slot = server.create_slot();
                let star_map_slot_id = star_map_slot.get_id();
                let (star_map_account_sender, star_map_account_receiver) = channel();
                
                Thread::spawn(move || {
                    server.listen("localhost:30000");
                });
                
                Thread::spawn(move || {
                    login::run_login_server(login_slot, star_map_slot_id, star_map_account_sender);
                });
                
                Thread::spawn(move || {
                    let mut star_map_server = StarMapServer::new(star_map_slot);
                    star_map_server.run(star_map_account_receiver);
                });
                
                // Connect to server
                let mut client = Client::new(ip_address.as_slice());

                let mut packet = OutPacket::new();
                packet.write(&LoginPacket{username: username, password: password});
                client.send(&packet);
                
                // Receive the ships from the server
                let mut packet = client.receive();
                let turn_time_milliseconds: u32 = packet.read().ok().expect("Failed to read turn time from server");
                let player_ship = packet.read().ok().expect("Failed to read player's ship");
                let ships = match packet.read() {
                    Ok(ships) => ships,
                    Err(e) => panic!("Unable to receive ships froms server: {}", e),
                };
                
                // Create the battle state
                let mut battle_context = BattleContext::new(ships);
                battle_context.add_ship(player_ship);
                let mut battle = ClientBattleState::new(client, battle_context);

                battle.run(window, gl, &asset_store, 5000 - (turn_time_milliseconds as i64));
            },
            MainMenuSelection::Tutorial => {                
                // Create the tutorial state
                let mut battle = TutorialState::new();

                battle.run(window, gl, &asset_store);
            },
            MainMenuSelection::Exit => {
                
            },
        }
    });
}