use ggez::{graphics, event, Context, GameResult};
use ggez::event::{Event, Keycode, MouseButton};
use ggez::graphics::{DrawParam, DrawMode, FontId, Image, Point2, Rect, Scale, TextCached};

use vec::Vec2f;

#[derive(PartialEq)]
enum MouseFocus {
    NoHover,
    Hover,
    Focus,
}

pub struct SpriteButton {
    image: Image,
    
    columns: u8,
    rows: u8,
    frame_w: f32,
    frame_h: f32,
    current_frame: u8,
    
    position: [f32; 2],
    size: [f32; 2],
    
    clicked: bool,
    mouse_focus: MouseFocus,
}

impl SpriteButton {
    pub fn new(
        ctx: &mut Context,
        path: &str, frames_per_row: u8, rows: u8, position: [f32; 2])
        -> GameResult<SpriteButton>
    {
        let image = Image::new(ctx, path)?;
        let (frame_w, frame_h) = (
            image.width() / (frames_per_row as u32),
            image.height() / (rows as u32)
        );
        
        Ok(SpriteButton {
            image: image,
            
            columns: frames_per_row,
            rows: rows,
            // Frame width and height represented here with the width of the
            // sprite sheet being 1.0
            frame_w: 1.0 / (frames_per_row as f32),
            frame_h: 1.0 / (rows as f32),
            current_frame: 0,
            
            position: position,
            size: [frame_w as f32, frame_h as f32],
            
            clicked: false,
            mouse_focus: MouseFocus::NoHover,
        })
    }
    
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let src_x = ((self.current_frame % self.columns) as f32) * self.frame_w;
        let src_y = ((self.current_frame / self.columns) as f32) * self.frame_h;

        graphics::draw_ex(
            ctx, &self.image,
            DrawParam {
                dest: Point2::new(self.position[0], self.position[1]),
                src: Rect::new(src_x, src_y, self.frame_w, self.frame_h),
                ..Default::default()
            });

        Ok(())
    }
    
    pub fn event(&mut self, e: &Event, mouse_pos: Vec2f) {
        use Event::*;

        match *e {
            MouseMotion { x, y, .. } => {
                let x = x as f32;
                let y = y as f32;
                if self.mouse_focus != MouseFocus::Focus {
                    if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                        y >= self.position[1] && y <= self.position[1]+self.size[1]
                    {
                        self.mouse_focus = MouseFocus::Hover;
                        self.current_frame = 1;
                    } else {
                        self.mouse_focus = MouseFocus::NoHover;
                        self.current_frame = 0;
                    }
                }
            },
            MouseButtonDown { mouse_btn, x, y, .. } => {
                if mouse_btn == MouseButton::Left {
                    let x = x as f32;
                    let y = y as f32;
                    if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                        y >= self.position[1] && y <= self.position[1]+self.size[1]
                    {
                        self.mouse_focus = MouseFocus::Focus;
                        self.current_frame = 2;
                    } else {
                        self.mouse_focus = MouseFocus::NoHover;
                        self.current_frame = 0;
                    }
                }
            }
            MouseButtonUp { mouse_btn, x, y, .. } => {
                if mouse_btn == MouseButton::Left {
                    let x = x as f32;
                    let y = y as f32;
                    if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                        y >= self.position[1] && y <= self.position[1]+self.size[1]
                    {
                        self.clicked = true;
                        self.mouse_focus = MouseFocus::Hover;
                        self.current_frame = 1;
                    } else {
                        self.mouse_focus = MouseFocus::NoHover;
                        self.current_frame = 0;
                    }
                }
            },
            _ => { }
        }
    }
    
    pub fn get_clicked(&mut self) -> bool {
        let clicked = self.clicked;
        self.clicked = false;
        clicked
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TextButton {
    pub text: TextCached,
    bg_color: [f32; 4],
    bg_color_hover: [f32; 4],
    bg_color_focus: [f32; 4],
    text_color: [f32; 4],
    
    position: [f32; 2],
    size: [f32; 2],
    
    clicked: bool,
    
    mouse_focus: MouseFocus,
}

impl TextButton {
    pub fn new(
        font: FontId, text: &str,
        font_size: f32, position: [f32; 2], size: [f32; 2])
        -> GameResult<TextButton>
    {
        Ok(TextButton {
            text: TextCached::new((text, font, Scale::uniform(font_size)))?,
            bg_color: [0.3, 1.0, 0.0, 1.0],
            bg_color_hover: [0.6, 1.0, 0.0, 1.0],
            bg_color_focus: [1.0, 0.0, 0.0, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            
            position: position,
            size: size,
            
            clicked: false,
            
            mouse_focus: MouseFocus::NoHover,
        })
    }
    
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let bg_color =
            match self.mouse_focus {
                MouseFocus::NoHover => self.bg_color,
                MouseFocus::Hover => self.bg_color_hover,
                MouseFocus::Focus => self.bg_color_focus,
            };
    
        // Draw background rectangle
        graphics::set_color(ctx, bg_color.into());
        graphics::rectangle(
            ctx, DrawMode::Fill,
            Rect::new(self.position[0], self.position[1], self.size[0], self.size[1]))?;
        
        // Draw text
        let buffer = (self.size[1] - (self.text.height(ctx) as f32)) / 2.0;
        let pos = Point2::new(
            self.position[0] + buffer,
            self.position[1] + buffer);
        graphics::set_color(ctx, self.text_color.into());
        graphics::draw(ctx, &self.text, pos, 0.0)?;
        graphics::set_color(ctx, [1.0; 4].into());

        Ok(())
    }

    pub fn event(&mut self, e: &Event, mouse_pos: Vec2f) {
        use Event::*;

        let x = mouse_pos.x as f32;
        let y = mouse_pos.y as f32;
        if self.mouse_focus != MouseFocus::Focus {
            if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                y >= self.position[1] && y <= self.position[1]+self.size[1]
            {
                self.mouse_focus = MouseFocus::Hover;
            } else {
                self.mouse_focus = MouseFocus::NoHover;
            }
        }

        match *e {
            MouseButtonDown { mouse_btn, .. } => {
                if mouse_btn == MouseButton::Left {
                    if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                        y >= self.position[1] && y <= self.position[1]+self.size[1]
                    {
                        self.mouse_focus = MouseFocus::Focus;
                    } else {
                        self.mouse_focus = MouseFocus::NoHover;
                    }
                }
            }
            MouseButtonUp { mouse_btn, .. } => {
                if mouse_btn == MouseButton::Left {
                    if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                        y >= self.position[1] && y <= self.position[1]+self.size[1]
                    {
                        self.clicked = true;
                        self.mouse_focus = MouseFocus::Hover;
                    } else {
                        self.mouse_focus = MouseFocus::NoHover;
                    }
                }
            },
            _ => { }
        }
    }
    
    pub fn get_clicked(&mut self) -> bool {
        let clicked = self.clicked;
        self.clicked = false;
        clicked
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TextBox {
    pub text: String,
    pub draw_text: TextCached,
    pub bg_color: [f32; 4],
    pub bg_color_hover: [f32; 4],
    pub bg_color_focus: [f32; 4],
    pub text_color: [f32; 4],
    
    position: [f32; 2],
    size: [f32; 2],
    
    pub has_focus: bool,
    pub hide_text: bool,
    
    mouse_focus: MouseFocus,
    
    cursor_position: u8,
}

impl TextBox {
    pub fn new(
        font: FontId, text: String, font_size: f32,
        position: [f32; 2], size: [f32; 2])
        -> GameResult<TextBox>
    {
        let mut draw_text = TextCached::new(text.as_ref())?;
        draw_text.set_font(font, Scale::uniform(font_size));
        Ok(TextBox {
            text: text,
            draw_text: draw_text,
            bg_color: [0.3, 0.3, 1.0, 1.0],
            bg_color_hover: [0.5, 0.5, 1.0, 1.0],
            bg_color_focus: [0.0, 1.0, 0.0, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            
            position: position,
            size: size,
            
            has_focus: false,
            hide_text: false,
            
            mouse_focus: MouseFocus::NoHover,
            
            cursor_position: 0,
        })
    }
    
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let bg_color =
            match self.mouse_focus {
                MouseFocus::NoHover => self.bg_color,
                MouseFocus::Hover => self.bg_color_hover,
                MouseFocus::Focus => self.bg_color_focus,
            };

        // Draw background rectangle
        graphics::set_color(ctx, bg_color.into());
        graphics::rectangle(
            ctx, DrawMode::Fill,
            Rect::new(self.position[0], self.position[1], self.size[0], self.size[1]))?;
        
        // Draw text
        let buffer = (self.size[1] - (self.draw_text.height(ctx) as f32)) / 2.0;
        let pos = Point2::new(
            self.position[0] + buffer,
            self.position[1] + buffer);
        graphics::set_color(ctx, self.text_color.into());
        graphics::draw(ctx, &self.draw_text, pos, 0.0)?;

        Ok(())
    }

    pub fn event(&mut self, e: &Event, mouse_pos: Vec2f) {
        use Event::*;

        let x = mouse_pos.x as f32;
        let y = mouse_pos.y as f32;
        if self.mouse_focus != MouseFocus::Focus {
            if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                y >= self.position[1] && y <= self.position[1]+self.size[1]
            {
                self.mouse_focus = MouseFocus::Hover;
            } else {
                self.mouse_focus = MouseFocus::NoHover;
            }
        }

        match *e {
            KeyDown { keycode: Some(keycode), .. } => {
                if self.has_focus {
                    if keycode == Keycode::Backspace {
                        self.text.pop();
                        self.on_text_update();
                    }
                }
            },
            MouseButtonDown { mouse_btn, .. } => {
                if mouse_btn == MouseButton::Left {
                    if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                        y >= self.position[1] && y <= self.position[1]+self.size[1]
                    {
                        self.has_focus = true;
                        self.mouse_focus = MouseFocus::Focus;
                    } else {
                        self.has_focus = false;
                        self.mouse_focus = MouseFocus::NoHover;
                    }
                }
            }
            TextInput { ref text, .. } => {
                if self.has_focus {
                    self.text.push_str(text);
                    self.on_text_update();
                }
            },
            _ => { }
        }
    }

    fn on_text_update(&mut self) {
        if !self.hide_text {
            self.draw_text.replace_fragment(0, self.text.as_ref());
        } else {
            let text: String = self.text.chars().map(|_| '*').collect();
            self.draw_text.replace_fragment(0, text.as_ref());
        };
    }
}
