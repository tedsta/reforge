use std::cell::RefCell;
use std::num::FromPrimitive;

use sdl2_window::Sdl2Window;
use event::{Events, GenericEvent, RenderArgs};
use graphics::{Context, ImageSize};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};

use asset_store::AssetStore;
use net::{Client, OutPacket};

#[derive(FromPrimitive)]
pub enum MainMenuSelection {
    SinglePlayer,
    Multiplayer,
    Tutorial,
    Exit,
}

pub struct MainMenu {
    selected: u8,
    done: bool,

    mouse_x: f64,
    mouse_y: f64,

    // Textures
    bg_texture: Texture,
    single_player_texture: Texture,
    multiplayer_texture: Texture,
    tutorial_texture: Texture,
    exit_texture: Texture,
}

impl MainMenu {
    pub fn new() -> MainMenu {
        MainMenu {
            selected: 0,
            done: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            bg_texture: Texture::from_path(&Path::new("content/textures/gui/main_menu.png")).unwrap(),
            single_player_texture: Texture::from_path(&Path::new("content/textures/gui/singleplayer.png")).unwrap(),
            multiplayer_texture: Texture::from_path(&Path::new("content/textures/gui/multiplayer.png")).unwrap(),
            tutorial_texture: Texture::from_path(&Path::new("content/textures/gui/tutorial.png")).unwrap(),
            exit_texture: Texture::from_path(&Path::new("content/textures/gui/exit.png")).unwrap(),
        }
    }

    pub fn run(mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, asset_store: &AssetStore) -> Option<MainMenuSelection> {
        let mut menu_selection = None;
    
        // Main loop
        for e in Events::new(window) {
            use event;
            use input;
            use event::*;

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
        use event::*;
        
        e.mouse_cursor(|x, y| {
            self.on_mouse_moved(x, y);
        });
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                Button::Mouse(button) => {
                    self.on_mouse_pressed(button);
                },
            }
        });
    }

    fn on_key_pressed(&mut self, key: keyboard::Key) {
        use input::keyboard::Key;
        match key {
            Key::Up if self.selected > 0 => { self.selected -= 1; },
            Key::Up if self.selected == 0 => { self.selected = 3; },
            Key::Down if self.selected < 3 => { self.selected += 1; },
            Key::Down if self.selected == 3 => { self.selected = 0; },
            Key::Return => { self.done = true; },
            _ => {},
        }
    }

    fn on_mouse_pressed(&mut self, button: mouse::MouseButton) {
        match button {
            mouse::MouseButton::Left => {
                if self.is_mouse_over_button() == 0 {
                    self.done = true;
                } else if self.is_mouse_over_button() == 1 {
                    self.done = true;
                } else if self.is_mouse_over_button() == 2 {
                    self.done = true;
                } else if self.is_mouse_over_button() == 3 {
                    self.done = true;
                } else {}
            },
            mouse::MouseButton::Right => {},
            _ => {},
        }
    }

    fn on_mouse_moved(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;

        self.selected = self.is_mouse_over_button();
    }

    fn is_mouse_over_button(&mut self) -> u8 {
        let (s_width, s_height) = self.single_player_texture.get_size();
        let (m_width, m_height) = self.multiplayer_texture.get_size();
        let (t_width, t_height) = self.tutorial_texture.get_size();
        let (e_width, e_height) = self.exit_texture.get_size();

        let mut selected: u8; // is the "button" selected
        selected = self.selected;

        if self.mouse_x > 550.0 && self.mouse_x < (550.0 + (s_width as f64)) && 
            self.mouse_y > 300.0 && self.mouse_y < (300.0 + (s_height as f64)) {
            selected = 0;
        } else if self.mouse_x > 550.0 && self.mouse_x < (550.0 + (s_width as f64)) && 
            self.mouse_y > 400.0 && self.mouse_y < (400.0 + (s_height as f64)) {
            selected = 1;
        } else if self.mouse_x > 550.0 && self.mouse_x < (550.0 + (t_width as f64)) && 
            self.mouse_y > 500.0 && self.mouse_y < (500.0 + (t_height as f64)) {
            selected = 2;
        } else if self.mouse_x > 550.0 && self.mouse_x < (550.0 + (e_width as f64)) && 
            self.mouse_y > 600.0 && self.mouse_y < (600.0 + (e_height as f64)) {
            selected = 3;
        }

        selected
    }

    fn draw(&mut self, context: &Context, gl: &mut Gl, asset_store: &AssetStore) {
        use quack::Set;
        use graphics::*;
        
        // Clear the screen
        clear([0.0; 4], gl);

        image(&self.bg_texture, context, gl);
        image(&self.single_player_texture, &context.trans(550.0, 300.0), gl);
        image(&self.multiplayer_texture, &context.trans(550.0, 400.0), gl);
        image(&self.tutorial_texture, &context.trans(550.0, 500.0), gl);
        image(&self.exit_texture, &context.trans(550.0, 600.0), gl);

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
                .draw(&self.tutorial_texture, &context.trans(550.0, 500.0), gl);
        }
        if self.selected == 3 {
            Image::new()
                .set(Color([1.0, 0.0, 0.0, 1.0]))
                .draw(&self.exit_texture, &context.trans(550.0, 600.0), gl);
        }
    }
}
