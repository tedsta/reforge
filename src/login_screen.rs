use ggez::{Context, GameResult};
use ggez::graphics::{self, FontId, Image, TextCached, TextFragment, Point2, Scale};
use ggez::event::Event;

use game_state::GameState;
use gui::{TextBox, TextButton};
//use login::LoginError;
use vec::{Vec2, Vec2f};

#[derive(Clone)]
pub enum LoginGuiAction {
    Login(String, String, String),
    Back,
}

pub struct LoginScreen {
    //pub login_error: Option<LoginError>,
    mouse_pos: Vec2f,

    bg_image: Image,

    // Labels
    username_lbl: TextCached,
    password_lbl: TextCached,
    server_lbl: TextCached,
    
    // Text boxes
    username_box: TextBox,
    password_box: TextBox,
    ip_box: TextBox,
    
    // Buttons
    back_button: TextButton,
    login_button: TextButton,
}

impl LoginScreen {
    pub fn new(ctx: &mut Context, font: FontId) -> GameResult<LoginScreen> {
        let mut password_box = TextBox::new(
            font, "".to_string(), 20.0, [600.0, 370.0], [300.0, 40.0])?;
        password_box.hide_text = true;
    
        Ok(LoginScreen {
            //login_error: None,
            mouse_pos: Vec2::new(0.0, 0.0),

            bg_image: Image::new(ctx, "/textures/gui/main_menu.png").unwrap(),

            username_lbl: TextCached::new(("Username", font, Scale::uniform(20.0)))?,
            password_lbl: TextCached::new(("Password", font, Scale::uniform(20.0)))?,
            server_lbl: TextCached::new(("Server", font, Scale::uniform(20.0)))?,
            
            username_box: TextBox::new(
                font, "user".to_string(), 24.0, [600.0, 300.0], [300.0, 40.0])?,
            password_box: password_box,
            ip_box: TextBox::new(
                font, "localhost".to_string(), 24.0, [600.0, 440.0], [300.0, 40.0])?,
            
            back_button: TextButton::new(
                font, "Back", 24.0, [450.0, 500.0], [150.0, 40.0])?,
            login_button: TextButton::new(
                font, "Login", 24.0, [610.0, 500.0], [150.0, 40.0])?,
        })
    }
}

impl GameState for LoginScreen {
    type Context = ();
    type Action = LoginGuiAction;

    fn event(&mut self, _gtx: &mut Self::Context, e: &Event) -> Option<Self::Action> {
        use Event::*;
        match *e {
            MouseMotion { x, y, .. } => {
                self.mouse_pos = Vec2::new(x as f64, y as f64);
            },
            _ => { },
        }

        // Handle text boxes
        self.username_box.event(e, self.mouse_pos);
        self.password_box.event(e, self.mouse_pos);
        self.ip_box.event(e, self.mouse_pos);
        
        // Handle buttons
        self.login_button.event(e, self.mouse_pos);
        self.back_button.event(e, self.mouse_pos);
        
        if self.back_button.get_clicked() {
            Some(LoginGuiAction::Back)
        } else if self.login_button.get_clicked() {
            Some(LoginGuiAction::Login(self.username_box.text.clone(),
                                       self.password_box.text.clone(),
                                       self.ip_box.text.clone()))
        } else {
            None
        }
    }

    fn draw(&mut self, _gtx: &mut Self::Context, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.bg_image, Point2::new(0.0, 0.0), 0.0);
        
        // Draw the username and password labels
        graphics::set_color(ctx, [1.0; 4].into());
        graphics::draw(ctx, &self.username_lbl, Point2::new(450.0, 310.0), 0.0)?;
        graphics::draw(ctx, &self.password_lbl, Point2::new(450.0, 380.0), 0.0)?;
        graphics::draw(ctx, &self.server_lbl, Point2::new(450.0, 450.0), 0.0)?;
        
        // Draw the text boxes
        self.username_box.draw(ctx)?;
        self.password_box.draw(ctx)?;
        self.ip_box.draw(ctx)?;
        
        // Draw the buttons
        self.back_button.draw(ctx)?;
        self.login_button.draw(ctx)?;
        
        // Draw error messages
        /*if let Some(login_error) = self.login_error {
            match login_error {
                LoginError::NoSuchAccount => {
                    let context = context.trans(910.0, 330.0);
                    Text::new_color([1.0, 0.0, 0.0, 1.0], 30).draw(
                        "User doesn't exist",
                        glyph_cache,
                        &context.draw_state, context.transform,
                        gl,
                    );
                },
                LoginError::AlreadyLoggedIn => {
                    let context = context.trans(910.0, 330.0);
                    Text::new_color([1.0, 0.0, 0.0, 1.0], 30).draw(
                        "User already logged in",
                        glyph_cache,
                        &context.draw_state, context.transform,
                        gl,
                    );
                },
                LoginError::WrongPassword => {
                    let context = context.trans(910.0, 400.0);
                    Text::new_color([1.0, 0.0, 0.0, 1.0], 30).draw(
                        "Incorrect password",
                        glyph_cache,
                        &context.draw_state, context.transform,
                        gl,
                    );
                },
            }
        }*/

        Ok(())
    }
}
