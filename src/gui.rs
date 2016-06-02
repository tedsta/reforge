use graphics::{Context, ImageSize};
use piston::input::*;
use opengl_graphics::{GlGraphics, Texture};
use opengl_graphics::glyph_cache::GlyphCache;
use std::path::Path;

#[derive(PartialEq)]
enum MouseFocus {
    NoHover,
    Hover,
    Focus,
}
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SpriteButton {
    texture: Texture,
    
    columns: u8,
    rows: u8,
    frame_width: u32,
    frame_height: u32,
    current_frame: u8,
    
    position: [f64; 2],
    size: [f64; 2],
    
    clicked: bool,
    mouse_focus: MouseFocus,
}

impl SpriteButton {
    pub fn new(path: &str, frames_per_row: u8, rows: u8, position: [f64; 2]) -> SpriteButton {
        let texture = Texture::from_path(&Path::new(path)).unwrap();
        let (texture_width, texture_height) = texture.get_size();
        let (frame_width, frame_height) = (texture_width/(frames_per_row as u32), texture_height/(rows as u32));
        
        SpriteButton {
            texture: texture,
            
            columns: frames_per_row,
            rows: rows,
            frame_width: frame_width,
            frame_height: frame_height,
            current_frame: 0,
            
            position: position,
            size: [frame_width as f64, frame_height as f64],
            
            clicked: false,
            mouse_focus: MouseFocus::NoHover,
        }
    }
    
    pub fn draw(&mut self, context: &Context, gl: &mut GlGraphics) {  
        use graphics::*;
    
        let source_x = ((self.current_frame % (self.columns)) as f64) * (self.frame_width as f64);
        let source_y = ((self.current_frame / (self.columns)) as f64) * (self.frame_height as f64);
        
        let source_end_x = source_x + (self.frame_width as f64);
        let source_end_y = source_y + (self.frame_height as f64);
        
        let mut context = context.trans(self.position[0], self.position[1]);
        
        Image::new()
            .src_rect([source_x as i32, source_y as i32, self.frame_width as i32, self.frame_height as i32])
            .draw(&self.texture, &context.draw_state, context.transform, gl);
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2]) {
        use piston::event_loop::*;
        
        e.mouse_cursor(|_, _| {
            if self.mouse_focus != MouseFocus::Focus {
                let x = mouse_pos[0];
                let y = mouse_pos[1];
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
        });
        e.press(|button| {
            if let Button::Mouse(button) = button {
                if button == mouse::MouseButton::Left {
                    let x = mouse_pos[0];
                    let y = mouse_pos[1];
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
        });
        e.release(|button| {
            if let Button::Mouse(button) = button {
                if button == mouse::MouseButton::Left {
                    let x = mouse_pos[0];
                    let y = mouse_pos[1];
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
            }
        });
    }
    
    pub fn get_clicked(&mut self) -> bool {
        let clicked = self.clicked;
        self.clicked = false;
        clicked
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TextButton {
    pub text: String,
    font_size: u32,
    bg_color: [f32; 4],
    bg_color_hover: [f32; 4],
    bg_color_focus: [f32; 4],
    text_color: [f32; 4],
    
    position: [f64; 2],
    size: [f64; 2],
    
    clicked: bool,
    
    mouse_focus: MouseFocus,
}

impl TextButton {
    pub fn new(text: String, font_size: u32, position: [f64; 2], size: [f64; 2]) -> TextButton {
        TextButton {
            text: text,
            font_size: font_size,
            bg_color: [0.3, 1.0, 0.0, 1.0],
            bg_color_hover: [0.6, 1.0, 0.0, 1.0],
            bg_color_focus: [1.0, 0.0, 0.0, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            
            position: position,
            size: size,
            
            clicked: false,
            
            mouse_focus: MouseFocus::NoHover,
        }
    }
    
    pub fn draw(&mut self, context: &Context, gl: &mut GlGraphics, glyph_cache: &mut GlyphCache) {
        use graphics::*;
    
        let bg_color =
            match self.mouse_focus {
                MouseFocus::NoHover => self.bg_color,
                MouseFocus::Hover => self.bg_color_hover,
                MouseFocus::Focus => self.bg_color_focus,
            };
    
        // Draw background rectangle
        Rectangle::new(bg_color)
            .draw(
                [self.position[0], self.position[1], self.size[0], self.size[1]],
                &context.draw_state, context.transform,
                gl
            );
        
        // Draw text
        {
            let buffer = (self.size[1] - (self.font_size as f64)) / 2.0;
            let context = context.trans(self.position[0] + buffer, self.position[1] + self.size[1] - buffer);
            Text::new_color(self.text_color, self.font_size).draw(
                self.text.as_str(),
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2]) {
        use piston::event_loop::*;
        
        e.mouse_cursor(|_, _| {
            if self.mouse_focus != MouseFocus::Focus {
                let x = mouse_pos[0];
                let y = mouse_pos[1];
                if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                    y >= self.position[1] && y <= self.position[1]+self.size[1]
                {
                    self.mouse_focus = MouseFocus::Hover;
                } else {
                    self.mouse_focus = MouseFocus::NoHover;
                }
            }
        });
        e.press(|button| {
            if let Button::Mouse(button) = button {
                if button == mouse::MouseButton::Left {
                    let x = mouse_pos[0];
                    let y = mouse_pos[1];
                    if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                        y >= self.position[1] && y <= self.position[1]+self.size[1]
                    {
                        self.mouse_focus = MouseFocus::Focus;
                    } else {
                        self.mouse_focus = MouseFocus::NoHover;
                    }
                }
            }
        });
        e.release(|button| {
            if let Button::Mouse(button) = button {
                if button == mouse::MouseButton::Left {
                    let x = mouse_pos[0];
                    let y = mouse_pos[1];
                    if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                        y >= self.position[1] && y <= self.position[1]+self.size[1]
                    {
                        self.clicked = true;
                        self.mouse_focus = MouseFocus::Hover;
                    } else {
                        self.mouse_focus = MouseFocus::NoHover;
                    }
                }
            }
        });
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
    pub font_size: u32,
    pub bg_color: [f32; 4],
    pub bg_color_hover: [f32; 4],
    pub bg_color_focus: [f32; 4],
    pub text_color: [f32; 4],
    
    position: [f64; 2],
    size: [f64; 2],
    
    pub has_focus: bool,
    pub hide_text: bool,
    
    mouse_focus: MouseFocus,
    
    cursor_position: u8,
}

impl TextBox {
    pub fn new(text: String, font_size: u32, position: [f64; 2], size: [f64; 2]) -> TextBox {
        TextBox {
            text: text,
            font_size: font_size,
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
        }
    }
    
    pub fn draw(&mut self, context: &Context, gl: &mut GlGraphics, glyph_cache: &mut GlyphCache) {
        use graphics::*;
        
        let bg_color =
            match self.mouse_focus {
                MouseFocus::NoHover => self.bg_color,
                MouseFocus::Hover => self.bg_color_hover,
                MouseFocus::Focus => self.bg_color_focus,
            };
    
        // Draw background rectangle
        Rectangle::new(bg_color)
            .draw(
                [self.position[0], self.position[1], self.size[0], self.size[1]],
                &context.draw_state, context.transform,
                gl
            );
        
        // Draw text
        {
            let text =
                if !self.hide_text {
                    self.text.clone()
                } else {
                    self.text.chars().map(|_| '*').collect()
                };
        
            let buffer = (self.size[1] - (self.font_size as f64)) / 2.0;
            let context = context.trans(self.position[0] + buffer, self.position[1] + self.size[1] - buffer);
            Text::new_color(self.text_color, self.font_size).draw(
                text.as_str(),
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2]) {
        use piston::event_loop::*;
        
        e.mouse_cursor(|_, _| {
            if self.mouse_focus != MouseFocus::Focus {
                let x = mouse_pos[0];
                let y = mouse_pos[1];
                if x >= self.position[0] && x <= self.position[0]+self.size[0] &&
                    y >= self.position[1] && y <= self.position[1]+self.size[1]
                {
                    self.mouse_focus = MouseFocus::Hover;
                } else {
                    self.mouse_focus = MouseFocus::NoHover;
                }
            }
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
                            self.mouse_focus = MouseFocus::Focus;
                        } else {
                            self.has_focus = false;
                            self.mouse_focus = MouseFocus::NoHover;
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
                _ => { },
            }
        });
        e.text(|text| {
            if self.has_focus {
                self.text.push_str(text);
            }
        });
    }
}
