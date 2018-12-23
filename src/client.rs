#![crate_name = "reforge_client"]
#![crate_type = "bin"]
#![feature(os)]
#![feature(alloc)]
#![feature(thread_sleep)]
#![feature(raw)]
#![feature(nll)]

extern crate float;
extern crate num;
extern crate rand;
extern crate time;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
//extern crate erased_serde;
extern crate bincode;

extern crate vecmath;
extern crate ggez;
extern crate gfx_core;
extern crate nalgebra as na;

use std::thread::{self, Thread};
use std::sync::Arc;
use std::sync::mpsc::channel;

use ggez::{event, graphics, Context, GameResult};
use ggez::graphics::{Font, FontId};
use ggez::event::{Event, Events};

use asset_store::AssetStore;
use battle_context::BattleContext;
use client_context::ReforgeClientContext;
//use sector_client::ClientBattleState;
use client_state::run_client_state_manager;
use login::{LoginPacket, LoginError};
use login_screen::{LoginScreen, LoginGuiAction};
use main_menu::{MainMenu, MainMenuSelection};
use module::ModelStore;
use net::{Client, OutPacket};
use sector_data::SectorData;
use star_map::StarMapServer;

// Server stuff
use net::Server;

mod ai;
mod asset_store;
mod battle_context;
mod sector_client;
mod chat;
mod client_action;
mod client_context;
mod client_state;
mod config;
mod game_state;
mod gui;
mod login;
mod login_screen;
mod main_menu;
mod module;
//mod nav_map_gui;
mod net;
mod packet_types;
mod sector_data;
mod sector_server;
mod ship;
mod sim;
mod sim_events;
mod sim_visuals;
mod space_gui;
mod sprite_sheet;
mod star_map;
mod util;
mod vec;


#[cfg(feature = "client")]
fn main() {
    let ref mut ctx = ggez::ContextBuilder::new("Reforge", "Feedus Games")
        .window_setup(ggez::conf::WindowSetup::default().title("Reforge"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(1280, 720))
        .build().expect("Failed to build ggez context");
    
    // Load our font
    let mut font: FontId = Font::new_glyph_font(ctx, "/fonts/OCRAStd.ttf").unwrap().into();
    
    let asset_store = AssetStore::new(ctx).expect("Failed to load assets");
    let model_store = Arc::new(ModelStore::new(ctx));

    //let music = sdl2_mixer::Music::from_file(&Path::new("content/audio/music/space.wav")).unwrap();
    //music.play(-1).ok().expect("Failed to play background music");
    
    // Start a local server
    let mut server = Server::new();
    let login_slot = server.create_slot();
    let star_map_slot = server.create_slot();
    let star_map_slot_id = star_map_slot.get_id();
    let (star_map_account_sender, star_map_account_receiver) = channel();
    let (logout_sender, logout_receiver) = channel();
    let login_model_store = model_store.clone();
    let star_map_model_store = login_model_store.clone();
    
    thread::Builder::new().name("server_master".to_string()).spawn(move || {
        server.listen("localhost:30000");
    });
    
    thread::Builder::new().name("login_server".to_string()).spawn(move || {
        login::run_login_server(
            login_model_store, login_slot, star_map_slot_id,
            star_map_account_sender, logout_receiver);
    });
    
    let mut star_map_server = StarMapServer::new(star_map_model_store, star_map_slot);
    thread::Builder::new().name("star_map_server".to_string()).spawn(move || {
        star_map_server.run(star_map_account_receiver, logout_sender);
    });

    graphics::set_background_color(ctx, [0.0, 0.0, 0.0, 0.0].into());
    match run_reforge_client(ctx, font, asset_store, model_store) {
        Err(e) => println!("Error: {}", e),
        Ok(_) => println!("Reforge exited cleanly"),
    }
}

fn run_reforge_client(
    ctx: &mut Context, font: FontId,
    asset_store: AssetStore, model_store: Arc<ModelStore>)
    -> GameResult<()>
{
    let mut main_menu = MainMenu::new(ctx).unwrap();
    let mut login_screen = LoginScreen::new(ctx, font).unwrap();

    loop {
        match game_state::run(&mut (), ctx, &mut main_menu)? {
            Some(MainMenuSelection::Multiplayer) => {
                match game_state::run(&mut (), ctx, &mut login_screen)? {
                    Some(LoginGuiAction::Login(username, password, ip_address)) => {
                        // Connect to server
                        let mut client = Client::new((ip_address+":30000").as_str());

                        let mut packet = OutPacket::new();
                        packet.write(&LoginPacket { username: username, password: password });
                        client.send(&packet);
                        
                        let mut login_result_packet = client.receive();
                        let login_result: Option<LoginError> = login_result_packet.read().unwrap();
                        
                        match login_result {
                            Some(login_error) => {
                                //login_screen.login_error = Some(login_error);
                            },
                            None => {
                                // Receive the star map
                                let mut packet = client.receive();
                                let sectors: Vec<SectorData> =
                                    packet.read().ok().expect("Failed to read star map");

                                let mut gtx = ReforgeClientContext {
                                    font, asset_store, model_store, client, sectors
                                };

                                run_client_state_manager(&mut gtx, ctx);
                                break;
                            },
                        }
                    },
                    Some(LoginGuiAction::Back) => {
                        // Go back to main menu
                        break;
                    },
                    None => { },
                }
            }
            Some(MainMenuSelection::Exit) => { break; }
            _ => { break; },
        }
    }

    Ok(())
}
