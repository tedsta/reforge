use std::cell::RefCell;
use std::num::FromPrimitive;

use sdl2_window::Sdl2Window;
use event::{Events, GenericEvent, RenderArgs};
use graphics::{Context, ImageSize};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use asset_store::AssetStore;
use net::{Client, OutPacket};

pub struct MainMenu {
    done: bool,

    mouse_x: f64,
    mouse_y: f64,

    // Textures
    bg_texture: Texture,
}

impl MainMenu {
    pub fn new() -> MainMenu {
        MainMenu {
            done: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            bg_texture: Texture::from_path(&Path::new("content/textures/gui/main_menu.png")).unwrap(),
        }
    }

    pub fn run(mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        // Main loop
        for e in Events::new(window) {
            use event;
            use input;
            use event::*;

            let e: event::Event<input::Input> = e;

            self.event(&e);

            // Render GUI
            e.render(|args: &RenderArgs| {
                gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
                    self.draw(&c, gl, glyph_cache);
                });
            });

            if self.done {
                break;
            }
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E) {
        use event::*;
        
        e.mouse_cursor(|x, y| {
            self.on_mouse_moved(x, y);
        });
        e.press(|button| {
            match button {
                Button::Mouse(button) => {
                    self.on_mouse_pressed(button);
                },
                _ => {},
            }
        });
    }

    fn on_mouse_pressed(&mut self, button: mouse::MouseButton) {
        match button {
            mouse::MouseButton::Left => {},
            mouse::MouseButton::Right => {},
            _ => {},
        }
    }

    fn on_mouse_moved(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use quack::Set;
        use graphics::*;
        
        // Clear the screen
        clear([0.0; 4], gl);

        image(&self.bg_texture, context, gl);
    }
}
