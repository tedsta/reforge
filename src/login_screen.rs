use std::cell::RefCell;
use std::num::FromPrimitive;

use sdl2_window::Sdl2Window;
use event::{Events, GenericEvent, RenderArgs};
use graphics::{Color, Context};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use asset_store::AssetStore;
use net::{Client, OutPacket};

pub struct LoginScreen {
    done: bool,

    mouse_x: f64,
    mouse_y: f64,

    // Textures
    bg_texture: Texture,
    
    // Buttons
    cancel_button: TextButton,
    login_button: TextButton,
}

impl LoginScreen {
    pub fn new() -> LoginScreen {
        LoginScreen {
            done: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            bg_texture: Texture::from_path(&Path::new("content/textures/gui/main_menu.png")).unwrap(),
            cancel_button: TextButton::new("Cancel".to_string(), 20, [300.0, 300.0], [90.0, 20.0]),
            login_button: TextButton::new("Login".to_string(), 20, [400.0, 300.0], [90.0, 20.0]),
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
        
        // Draw the buttons
        self.cancel_button.draw(context, gl, glyph_cache);
        self.login_button.draw(context, gl, glyph_cache);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct TextButton {
    text: String,
    font_size: u32,
    bg_color: [f32; 4],
    text_color: [f32; 4],
    
    position: [f64; 2],
    size: [f64; 2],
}

impl TextButton {
    pub fn new(text: String, font_size: u32, position: [f64; 2], size: [f64; 2]) -> TextButton {
        TextButton {
            text: text,
            font_size: font_size,
            bg_color: [0.3, 0.9, 0.0, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            
            position: position,
            size: size,
        }
    }
    
    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use graphics::*;
        use graphics::text::Text;
    
        Rectangle::new(self.bg_color)
            .draw([self.position[0], self.position[1], self.size[0], self.size[1]], context, gl);
        
        Text::colored(self.text_color, self.font_size).draw(
            self.text.as_slice(),
            glyph_cache,
            &context.trans(self.position[0] + 5.0, self.position[1] + 5.0),
            gl,
        );
    }
}
