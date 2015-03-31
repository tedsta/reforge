use std::rc::Rc;
use std::cell::RefCell;

use sdl2_window::Sdl2Window;
use event::{Events, GenericEvent};
use graphics::{Color, Context};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use gui::{TextBox, TextButton};

pub struct LoginScreen {
    done: bool,
    login_info: Option<(String, String)>,

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
            login_info: None,
            
            mouse_x: 0.0,
            mouse_y: 0.0,
            
            username_box: TextBox::new("user".to_string(), 20, [600.0, 300.0], [300.0, 40.0]),
            password_box: TextBox::new("pass".to_string(), 20, [600.0, 370.0], [300.0, 40.0]),
            
            cancel_button: TextButton::new("Cancel".to_string(), 20, [450.0, 500.0], [150.0, 40.0]),
            login_button: TextButton::new("Login".to_string(), 20, [610.0, 500.0], [150.0, 40.0]),
        }
    }

    pub fn run(mut self, window: &Rc<RefCell<Sdl2Window>>, gl: &mut Gl, glyph_cache: &mut GlyphCache, bg_texture: &Texture) -> Option<(String, String)> {
        // Main loop
        for e in Events::new(window.clone()) {
            use event;
            use input;
            use event::*;

            let e: event::Event<input::Input> = e;

            self.event(&e);

            // Render GUI
            e.render(|args| {
                gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
                    self.draw(&c, gl, glyph_cache, bg_texture);
                });
            });

            if self.done {
                break;
            }
        }
        
        self.login_info
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
        
        // Handle buttons
        self.login_button.event(e, [self.mouse_x, self.mouse_y]);
        self.cancel_button.event(e, [self.mouse_x, self.mouse_y]);
        
        if self.cancel_button.get_clicked() {
            self.done = true;
            self.login_info = None;
        }
        
        if self.login_button.get_clicked() {
            self.done = true;
            self.login_info = Some((self.username_box.text.clone(), self.password_box.text.clone()));
        }
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

        image(bg_texture, context.transform, gl);
        
        // Draw the username and password labels
        {
            let context = context.trans(400.0, 330.0);
            Text::colored([1.0; 4], 30).draw(
                "Username",
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
        }
        
        {
            let context = context.trans(400.0, 400.0);
            Text::colored([1.0; 4], 30).draw(
                "Password",
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
        }
        
        // Draw the text boxes
        self.username_box.draw(context, gl, glyph_cache);
        self.password_box.draw(context, gl, glyph_cache);
        
        // Draw the buttons
        self.cancel_button.draw(context, gl, glyph_cache);
        self.login_button.draw(context, gl, glyph_cache);
    }
}