#![crate_name = "reforge_client"]
#![crate_type = "bin"]
#![feature(box_syntax)]
#![feature(rand)]
#![feature(core)]
#![feature(os)]
#![feature(io)]
#![feature(old_io)]
#![feature(alloc)]
#![feature(collections)]
#![feature(std_misc)]

extern crate bincode;
extern crate time;
extern crate rustc_serialize;

// Piston stuff
extern crate sdl2;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate graphics;
extern crate event;
extern crate input;
extern crate quack;
extern crate sdl2_mixer;
extern crate shader_version;
extern crate vecmath;
extern crate window;

use std::os;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use std::thread::{Builder, Thread};
use std::sync::mpsc::channel;

use sdl2_window::Sdl2Window;
use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;
use window::WindowSettings;

use asset_store::AssetStore;
use battle_state::BattleContext;
use battle_type::BattleType;
use client_battle_state::ClientBattleState;
use client_state::run_client_state_manager;
use login::LoginPacket;
use login_screen::LoginScreen;
use main_menu::{MainMenu, MainMenuSelection};
use net::{Client, OutPacket};
use star_map_gui::StarMapGui;
use tutorial_state::TutorialState;

// Server stuff
use net::Server;
use star_map_server::StarMapServer;

#[macro_use]
mod util;

mod ai;
mod asset_store;
mod battle_state;
mod battle_type;
mod client_battle_state;
mod client_state;
mod gui;
mod login;
mod login_screen;
mod main_menu;
mod module;
mod net;
mod sector_data;
mod sector_state;
mod ship;
mod sim;
mod sim_events;
mod sim_visuals;
mod space_gui;
mod sprite_sheet;
mod star_map_gui;
mod tutorial_state;
mod vec;

// server stuff
mod star_map_server;

#[cfg(feature = "client")]
fn main () {
    let opengl = shader_version::OpenGL::_3_2;
    
    // Create an SDL window.
    let window = Sdl2Window::new(
        opengl,
        WindowSettings {
            title: "Reforge".to_string(),
            size: [1280, 720],
            samples: 0,
            fullscreen: false,
            exit_on_esc: true,
        }
    );
    
    // Initialize SDL mixer
    sdl2::init(sdl2::INIT_AUDIO | sdl2::INIT_TIMER);
    sdl2_mixer::init(sdl2_mixer::INIT_MP3 | sdl2_mixer::INIT_FLAC |
        sdl2_mixer::INIT_MOD | sdl2_mixer::INIT_FLUIDSYNTH |
        sdl2_mixer::INIT_MODPLUG | sdl2_mixer::INIT_OGG);
    
    // TODO: 0x8010 is SDL_audio flag
    sdl2_mixer::open_audio(sdl2_mixer::DEFAULT_FREQUENCY, 0x8010u16, 2, 1024).ok().expect("Failed to initialize SDL2 mixer");
    sdl2_mixer::allocate_channels(512);
    
    // Create GL device
    let mut gl = Gl::new(opengl);
    
    // Load our font
    let mut glyph_cache = GlyphCache::new(&Path::new("content/fonts/8bit.ttf")).unwrap();
    
    // Create the asset store
    let asset_store = AssetStore::new();

    // Wrap window in RefCell
    let window = Rc::new(RefCell::new(window));
    
    let music = sdl2_mixer::Music::from_file(&Path::new("content/audio/music/space.wav")).unwrap();
    
    music.play(-1);
    
    // Create main menu
    let mut main_menu = MainMenu::new();
    main_menu.run(&window, &mut gl, |window, gl, menu_bg, selection| {
        match selection {
            MainMenuSelection::Multiplayer => {
                if let Some((username, password)) = LoginScreen::new().run(&window, gl, &mut glyph_cache, menu_bg) {
                    // Check for IP address in args
                    /*
                    let mut ip_address =
                        if os::args().len() > 1 {
                            os::args()[1].clone()
                        } else {
                            prisize!("IP Address: ");
                            String::from_str(
                                io::stdin().read_line()
                                    .ok().expect("Failed to read IP address")
                                    .trim_left()
                            )
                        };
                    ip_address.push_str(":30000"); // Add the port to the end of the address
                    */
                    let ip_address = String::from_str("localhost:30000");
                    //let ip_address = String::from_str("104.131.129.181:30000");
                    
                    // Start a local server
                    let mut server = Server::new();
                    let login_slot = server.create_slot();
                    let star_map_slot = server.create_slot();
                    let star_map_slot_id = star_map_slot.get_id();
                    let (star_map_account_sender, star_map_account_receiver) = channel();
                    
                    Builder::new().name("server_master".to_string()).spawn(move || {
                        server.listen("localhost:30000");
                    });
                    
                    Builder::new().name("login_server".to_string()).spawn(move || {
                        login::run_login_server(login_slot, star_map_slot_id, star_map_account_sender);
                    });
                    
                    Builder::new().name("star_map_server".to_string()).spawn(move || {
                        let mut star_map_server = StarMapServer::new(star_map_slot);
                        star_map_server.run(star_map_account_receiver);
                    });
                    
                    // Connect to server
                    let mut client = Client::new(ip_address.as_slice());

                    let mut packet = OutPacket::new();
                    packet.write(&LoginPacket{username: username, password: password});
                    client.send(&packet);
                    
                    run_client_state_manager(&window, gl, &mut glyph_cache, &asset_store, client);
                }
            },
            MainMenuSelection::Tutorial => {                
                // Create the tutorial state
                let mut battle = TutorialState::new();

                battle.run(&window, gl, &mut glyph_cache, &asset_store);
            },
            MainMenuSelection::Exit => {
                
            },
        }
    });
    
    sdl2_mixer::Music::halt();
    sdl2_mixer::quit();
}