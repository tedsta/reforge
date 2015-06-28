#![crate_name = "reforge_client"]
#![crate_type = "bin"]
#![feature(box_syntax)]
#![feature(rand)]
#![feature(core)]
#![feature(os)]
#![feature(io)]
#![feature(old_io)]
#![feature(alloc)]
#![feature(thread_sleep)]
#![feature(collections)]
#![feature(std_misc)]
#![feature(convert)]
#![feature(collections_drain)]
#![feature(duration)]
#![feature(reflect_marker)]
#![feature(raw)]
#![feature(drain)]
#![feature(rc_unique)]
#![feature(os)]
#![feature(std_misc)]
#![feature(path_ext)]

extern crate bincode;
extern crate float;
extern crate num;
extern crate rand;
extern crate rustc_serialize;
extern crate time;

// Piston stuff
extern crate sdl2;
extern crate piston;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate graphics;
extern crate sdl2_mixer;
extern crate vecmath;

use std::os;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use std::thread::{Builder, Thread};
use std::sync::mpsc::channel;

use glutin_window::GlutinWindow;
use opengl_graphics::{
    GlGraphics,
    OpenGL,
};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::window::{WindowSettings, Size};

use asset_store::AssetStore;
use battle_context::BattleContext;
use battle_type::BattleType;
use sector_client::ClientBattleState;
use client_state::run_client_state_manager;
use login::{LoginPacket, LoginError};
use login_screen::{LoginScreen, LoginGuiAction};
use main_menu::{MainMenu, MainMenuSelection};
use module::ModelStore;
use net::{Client, OutPacket};
use star_map::StarMapServer;

// Server stuff
use net::Server;

mod ai;
mod asset_store;
mod battle_context;
mod battle_type;
mod sector_client;
mod chat;
mod client_action;
mod client_state;
mod config;
mod gui;
mod login;
mod login_screen;
mod main_menu;
mod module;
mod net;
mod no_encode;
mod packet_types;
mod sector_data;
mod sector_server;
mod ship;
mod sim;
mod sim_events;
mod sim_visuals;
mod space_gui;
//mod sprite_mgr;
mod sprite_sheet;
mod star_map;
mod vec;

#[cfg(feature = "client")]
fn main () {
    let opengl = OpenGL::_3_0;
    
    // Create an window.
    let window = GlutinWindow::new(
        opengl,
        WindowSettings::new("Reforge".to_string(), Size { width: 1280, height: 720 }),
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
    let mut gl = GlGraphics::new(opengl);
    
    // Load our font
    let mut glyph_cache = GlyphCache::new(&Path::new("content/fonts/8bit.ttf")).unwrap();
    
    // Create the asset store
    let ref asset_store = AssetStore::new();
    
    // Create the module model store
    let ref model_store = ModelStore::new();

    // Wrap window in RefCell
    let window = Rc::new(RefCell::new(window));
    
    let music = sdl2_mixer::Music::from_file(&Path::new("content/audio/music/space.wav")).unwrap();
    
    music.play(-1).ok().expect("Failed to play background music");
    
    // Start a local server
    let mut server = Server::new();
    let login_slot = server.create_slot();
    let star_map_slot = server.create_slot();
    let star_map_slot_id = star_map_slot.get_id();
    let (star_map_account_sender, star_map_account_receiver) = channel();
    let (logout_sender, logout_receiver) = channel();
    
    Builder::new().name("server_master".to_string()).spawn(move || {
        server.listen("localhost:30000");
    });
    
    Builder::new().name("login_server".to_string()).spawn(move || {
        login::run_login_server(login_slot, star_map_slot_id, star_map_account_sender, logout_receiver);
    });
    
    let mut star_map_server = StarMapServer::new(star_map_slot);
    Builder::new().name("star_map_server".to_string()).spawn(move || {
        star_map_server.run(star_map_account_receiver, logout_sender);
    });
    
    // Create main menu
    let mut main_menu = MainMenu::new();
    main_menu.run(&window, &mut gl, |window, gl, menu_bg, selection| {
        match selection {
            MainMenuSelection::Multiplayer => {
                let mut login_screen = LoginScreen::new();
            
                loop {
                    match login_screen.run(&window, gl, &mut glyph_cache, menu_bg) {
                        LoginGuiAction::Login(username, password, ip_address) => {
                            // Connect to server
                            let mut client = Client::new((ip_address+":30000").as_str());

                            let mut packet = OutPacket::new();
                            packet.write(&LoginPacket{username: username, password: password});
                            client.send(&packet);
                            
                            let mut login_result_packet = client.receive();
                            let login_result: Option<LoginError> = login_result_packet.read().unwrap();
                            
                            match login_result {
                                Some(login_error) => {
                                    login_screen.login_error = Some(login_error);
                                },
                                None => {
                                    run_client_state_manager(&window, gl, &mut glyph_cache, asset_store, model_store, client);
                                    break;
                                },
                            }
                        },
                        LoginGuiAction::Back => {
                            break;
                        },
                    }
                }
                
                true
            },
            MainMenuSelection::Exit => { false },
        }
    });
    
    sdl2_mixer::Music::halt();
    sdl2_mixer::quit();
}
