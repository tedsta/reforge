use std::cell::RefCell;

use sdl2_window::Sdl2Window;
use event::{Events, GenericEvent, RenderArgs};
use graphics::Context;
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};

use asset_store::AssetStore;
use net::{Client, OutPacket};

#[deriving(FromPrimitive)]
pub enum MainMenuSelection {
    SinglePlayer,
    Multiplayer,
    Exit,
}

pub struct MainMenu {
    selected: u8,
    done: bool,

    // Textures
    logo_texture: Texture,
    single_player_texture: Texture,
    multiplayer_texture: Texture,
    exit_texture: Texture,
}

impl MainMenu {
    pub fn new() -> MainMenu {
        MainMenu {
            selected: 0,
            done: false,
            logo_texture: Texture::from_path(&Path::new("content/textures/gui/logo.png")).unwrap(),
            single_player_texture: Texture::from_path(&Path::new("content/textures/gui/singleplayer.png")).unwrap(),
            multiplayer_texture: Texture::from_path(&Path::new("content/textures/gui/multiplayer.png")).unwrap(),
            exit_texture: Texture::from_path(&Path::new("content/textures/gui/exit.png")).unwrap(),
        }
    }

    pub fn run(mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, asset_store: &AssetStore) -> MainMenuSelection {
        // Main loop
        for e in Events::new(window) {
            use event::*;

            self.event(&e);

            // Render GUI
            e.render(|args| {
                gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
                    self.draw(&c, gl, asset_store);
                });
            });

            if self.done { break; }
        }

        FromPrimitive::from_u8(self.selected).expect("invalid MainMenuSelection")
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E) {
        use event::*;
        
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                _ => {},
            }
        });
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
        use input::keyboard::Key;
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
        use current::Set;
        use graphics::*;
        
        // Clear the screen
        clear([0.0, ..4], gl);

        image(&self.logo_texture, &context.trans(350.0, 50.0), gl);
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
