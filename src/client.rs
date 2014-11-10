#![crate_name = "reforge_client"]
#![desc = "reforge awesome mmo client"]
#![crate_type = "bin"]
#![feature(macro_rules)]
#![feature(default_type_params)]

extern crate bincode;
extern crate time;
extern crate serialize;

// Piston stuff
extern crate graphics;
extern crate piston;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate shader_version;
extern crate event;

use sdl2_window::Sdl2Window;
use opengl_graphics::Gl;
use shader_version::opengl::OpenGL_3_0;

use std::cell::RefCell;
use piston::{
    RenderArgs,
    UpdateArgs
};

use graphics::{
    Context,
    AddRectangle,
    AddColor,
    Draw,
    RelativeTransform2d,
};

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

fn main () {
    // https://github.com/jeremyletang/rust-sfml/issues/37
    //unsafe { ::std::rt::stack::record_sp_limit(0); }
    
    let opengl = pistion::shader_version::opengl::OpenGL_3_0;
    
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
    let mut client = Client::new("127.0.0.1:30000");
    
    // Receive the battle context from the server
    let mut packet = client.receive();
    let context = match packet.read() {
        Ok(context) => context,
        Err(e) => panic!("Unable to receive battle context froms server: {}", e),
    };
    
    // Create the battle state
    let mut battle = ClientBattleState::new(client, context);

    battle.run(&mut window, &mut gl, &asset_store);
}