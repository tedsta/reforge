use std::cell::RefCell;
use std::num::FromPrimitive;

use sdl2_window::Sdl2Window;
use piston::event::{Events, GenericEvent, RenderArgs};
use piston::graphics::Context;
use piston::input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};

use asset_store::AssetStore;
use net::{Client, OutPacket};

#[derive(FromPrimitive)]
pub enum MainMenuSelection {
    SinglePlayer,
    Multiplayer,
    Exit,
}

pub struct MainMenu {
    selected: u8,
    done: bool,

    // Textures
    bg_texture: Texture,
    single_player_texture: Texture,
    multiplayer_texture: Texture,
    exit_texture: Texture,
}

impl MainMenu {
    pub fn new() -> MainMenu {
        MainMenu {
            selected: 0,
            done: false,
            bg_texture: Texture::from_path(&Path::new("content/textures/gui/main_menu.png")).unwrap(),
            single_player_texture: Texture::from_path(&Path::new("content/textures/gui/singleplayer.png")).unwrap(),
            multiplayer_texture: Texture::from_path(&Path::new("content/textures/gui/multiplayer.png")).unwrap(),
            exit_texture: Texture::from_path(&Path::new("content/textures/gui/exit.png")).unwrap(),
        }
    }

    pub fn run(mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, asset_store: &AssetStore) -> Option<MainMenuSelection> {
        let mut menu_selection = None;
    
        // Main loop
        for e in Events::new(window) {
            use piston::event;
            use piston::input;
            use piston::event::*;

            let e: event::Event<input::Input> = e;

            self.event(&e);

            // Render GUI
            e.render(|&mut: args: &RenderArgs| {
                gl.draw([0, 0, args.width as i32, args.height as i32], |: c, gl| {
                    self.draw(&c, gl, asset_store);
                });
            });

            if self.done {
                menu_selection = Some(self.selected);
                break;
            }
        }

        menu_selection.map(|x| FromPrimitive::from_u8(x).expect("invalid MainMenuSelection"))
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E) {
        use piston::event::*;
        
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                _ => {},
            }
        });
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
        use piston::input::keyboard::Key;
        match key {
            Key::Up if self.selected > 0 => { self.selected -= 1; },
            Key::Up if self.selected == 0 => { self.selected = 2; },
            Key::Down if self.selected < 2 => { self.selected += 1; },
            Key::Down if self.selected == 2 => { self.selected = 0; },
            Key::Return => { self.done = true; },
            _ => {},
        }
    }

    fn draw(&mut self, context: &Context, gl: &mut Gl, asset_store: &AssetStore) {
        use piston::quack::Set;
        use piston::graphics::*;
        
        // Clear the screen
        clear([0.0; 4], gl);

        image(&self.bg_texture, context, gl);
        image(&self.single_player_texture, &context.trans(550.0, 300.0), gl);
        image(&self.multiplayer_texture, &context.trans(550.0, 400.0), gl);
        image(&self.exit_texture, &context.trans(550.0, 500.0), gl);

        if self.selected == 0 {
            Image::new()
                .set(Color([1.0, 0.0, 0.0, 1.0]))
                .draw(&self.single_player_texture, &context.trans(550.0, 300.0), gl);
        }
        if self.selected == 1 {
            Image::new()
                .set(Color([1.0, 0.0, 0.0, 1.0]))
                .draw(&self.multiplayer_texture, &context.trans(550.0, 400.0), gl);
        }
        if self.selected == 2 {
            Image::new()
                .set(Color([1.0, 0.0, 0.0, 1.0]))
                .draw(&self.exit_texture, &context.trans(550.0, 500.0), gl);
        }
    }
}
