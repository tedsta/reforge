use ggez::{Context, GameResult};
use ggez::graphics::{self, Image, Point2};
use ggez::event::{Event, Keycode, MouseButton};

use game_state::GameState;

#[derive(PartialEq)]
pub enum MainMenuSelection {
    Multiplayer,
    Exit,
}

pub struct MainMenu {
    selected: u8,

    mouse_x: f64,
    mouse_y: f64,

    // Textures
    bg_texture: Image,
    multiplayer_texture: Image,
    exit_texture: Image,
}

impl MainMenu {
    pub fn new(ctx: &mut Context) -> GameResult<MainMenu> {
        Ok(MainMenu {
            selected: 0,
            mouse_x: 0.0,
            mouse_y: 0.0,

            bg_texture: Image::new(ctx, "/textures/gui/main_menu.png")?,
            multiplayer_texture: Image::new(ctx, "/textures/gui/multiplayer.png")?,
            exit_texture: Image::new(ctx, "/textures/gui/exit.png")?,
        })
    }

    pub fn on_key_pressed(&mut self, key: Keycode) -> Option<MainMenuSelection> {
        match key {
            Keycode::Up if self.selected > 0 => { self.selected -= 1; },
            Keycode::Up if self.selected == 0 => { self.selected = 1; },
            Keycode::Down if self.selected < 1 => { self.selected += 1; },
            Keycode::Down if self.selected == 1 => { self.selected = 0; },
            Keycode::Return => { return Some(self.get_selection()); },
            _ => { },
        }

        None
    }

    pub fn on_mouse_left_pressed(&mut self) -> Option<MainMenuSelection> {
        if self.is_mouse_over_button().is_some() {
            Some(self.get_selection())
        } else {
            None
        }
    }

    pub fn on_mouse_moved(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;

        if let Some(s) = self.is_mouse_over_button() {
            self.selected = s;
        }
    }

    fn get_selection(&self) -> MainMenuSelection {
        match self.selected {
            0 => MainMenuSelection::Multiplayer,
            1 => MainMenuSelection::Exit,
            _ => panic!("Invalid main menu selection"),
        }
    }

    fn is_mouse_over_button(&mut self) -> Option<u8> {
        let m_width = self.multiplayer_texture.width();
        let m_height = self.multiplayer_texture.height();
        let e_width = self.exit_texture.width();
        let e_height = self.exit_texture.height();

        let mut selected: u8; // is the "button" selected
        selected = self.selected;

        if self.mouse_x > 550.0 && self.mouse_x < (550.0 + (m_width as f64)) && 
            self.mouse_y > 300.0 && self.mouse_y < (300.0 + (m_height as f64)) {
            selected = 0;
            Some(0)
        } else if self.mouse_x > 550.0 && self.mouse_x < (550.0 + (e_width as f64)) && 
            self.mouse_y > 400.0 && self.mouse_y < (400.0 + (e_height as f64)) {
            selected = 1;
            Some(1)
        } else {
            None
        }
    }
}

impl GameState for MainMenu {
    type Context = ();
    type Action = MainMenuSelection;

    fn event(&mut self, _gtx: &mut Self::Context, e: &Event) -> Option<Self::Action> {
        use Event::*;

        match *e {
            KeyDown { keycode: Some(keycode), .. } => {
                self.on_key_pressed(keycode)
            }
            MouseMotion { x, y, .. } => {
                self.on_mouse_moved(x as f64, y as f64);
                None
            }
            MouseButtonUp { mouse_btn, .. } => {
                match mouse_btn {
                    MouseButton::Left => self.on_mouse_left_pressed(),
                    _ => { None },
                }
            }
            _ => None,
        }
    }

    fn draw(&mut self, _gtx: &mut Self::Context, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.bg_texture, Point2::new(0.0, 0.0), 0.0)?;
        //graphics::draw(ctx, &self.multiplayer_texture, Point2::new(550.0, 300.0), 0.0)?;
        graphics::draw_ex(
            ctx, &self.multiplayer_texture, graphics::DrawParam {
                dest: Point2::new(550.0, 300.0),
                src: graphics::Rect::new(0.0, 0.0, 1.0, 1.0),
                ..Default::default()
            })?;
        graphics::draw(ctx, &self.exit_texture, Point2::new(550.0, 400.0), 0.0)?;

        // Draw selected text
        graphics::set_color(ctx, [1.0, 0.0, 0.0, 1.0].into());
        if self.selected == 0 {
            graphics::draw(ctx, &self.multiplayer_texture, Point2::new(550.0, 300.0), 0.0)?;
        }
        if self.selected == 1 {
            graphics::draw(ctx, &self.exit_texture, Point2::new(550.0, 400.0), 0.0)?;
        }
        graphics::set_color(ctx, [1.0, 1.0, 1.0, 1.0].into());

        Ok(())
    }
}
