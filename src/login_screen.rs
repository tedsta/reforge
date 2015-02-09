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
    
    // Text boxes
    username_box: TextBox,
    password_box: TextBox,
    
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
            
            username_box: TextBox::new("user".to_string(), 20, [600.0, 300.0], [300.0, 40.0]),
            password_box: TextBox::new("pass".to_string(), 20, [600.0, 370.0], [300.0, 40.0]),
            
            cancel_button: TextButton::new("Cancel".to_string(), 20, [450.0, 500.0], [150.0, 40.0]),
            login_button: TextButton::new("Login".to_string(), 20, [610.0, 500.0], [150.0, 40.0]),
        }
    }

    pub fn run(mut self, window: &RefCell<Sdl2Window>, gl: &mut Gl, glyph_cache: &mut GlyphCache, bg_texture: &Texture) {
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
                    self.draw(&c, gl, glyph_cache, bg_texture);
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
        
        // Handle text boxes
        self.username_box.event(e, [self.mouse_x, self.mouse_y]);
        self.password_box.event(e, [self.mouse_x, self.mouse_y]);
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

    fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache, bg_texture: &Texture) {
        use quack::Set;
        use graphics::*;
        use graphics::text::Text;
        
        // Clear the screen
        clear([0.0; 4], gl);

        image(bg_texture, context, gl);
        
        // Draw the username and password labels
        Text::colored([1.0; 4], 30).draw(
            "Username",
            glyph_cache,
            &context.trans(400.0, 330.0),
            gl,
        );
        
        Text::colored([1.0; 4], 30).draw(
            "Password",
            glyph_cache,
            &context.trans(400.0, 400.0),
            gl,
        );
        
        // Draw the text boxes
        self.username_box.draw(context, gl, glyph_cache);
        self.password_box.draw(context, gl, glyph_cache);
        
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
    
        // Draw background rectangle
        Rectangle::new(self.bg_color)
            .draw([self.position[0], self.position[1], self.size[0], self.size[1]], context, gl);
        
        // Draw text
        let buffer = (self.size[1] - (self.font_size as f64)) / 2.0;
        Text::colored(self.text_color, self.font_size).draw(
            self.text.as_slice(),
            glyph_cache,
            &context.trans(self.position[0] + buffer, self.position[1] + self.size[1] - buffer),
            gl,
        );
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct TextBox {
    text: String,
    font_size: u32,
    bg_color: [f32; 4],
    text_color: [f32; 4],
    
    position: [f64; 2],
    size: [f64; 2],
    
    pub has_focus: bool,
}

impl TextBox {
    pub fn new(text: String, font_size: u32, position: [f64; 2], size: [f64; 2]) -> TextBox {
        TextBox {
            text: text,
            font_size: font_size,
            bg_color: [0.3, 0.9, 0.0, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            
            position: position,
            size: size,
            
            has_focus: false,
        }
    }
    
    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use graphics::*;
        use graphics::text::Text;
    
        // Draw background rectangle
        Rectangle::new(self.bg_color)
            .draw([self.position[0], self.position[1], self.size[0], self.size[1]], context, gl);
        
        // Draw text
        let buffer = (self.size[1] - (self.font_size as f64)) / 2.0;
        Text::colored(self.text_color, self.font_size).draw(
            self.text.as_slice(),
            glyph_cache,
            &context.trans(self.position[0] + buffer, self.position[1] + self.size[1] - buffer),
            gl,
        );
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2]) {
        use event::*;
        
        e.mouse_cursor(|_, _| {
            // TODO
        });
        e.press(|button| {
            match button {
                Button::Mouse(button) => {
                    if button == mouse::MouseButton::Left {
                        let x = mouse_pos[0];
                        let y = mouse_pos[1];
                        if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                            y >= self.position[1] && y <= self.position[1]+self.size[1]
                        {
                            self.has_focus = true;
                        } else {
                            self.has_focus = false;
                        }
                    }
                },
                Button::Keyboard(key) => {
                    if self.has_focus {
                        if key == keyboard::Key::Backspace {
                            self.text.pop();
                        }
                    }
                },
            }
        });
        e.text(|text| {
            if self.has_focus {
                self.text.push_str(text);
            }
        });
    }
}
