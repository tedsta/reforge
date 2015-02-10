use event::{GenericEvent};
use graphics::{Context};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl};
use opengl_graphics::glyph_cache::GlyphCache;

#[derive(PartialEq)]
enum MouseFocus {
    NoHover,
    Hover,
    Focus,
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
    
    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use graphics::*;
        use graphics::text::Text;
    
        let bg_color =
            match self.mouse_focus {
                MouseFocus::NoHover => self.bg_color,
                MouseFocus::Hover => self.bg_color_hover,
                MouseFocus::Focus => self.bg_color_focus,
            };
    
        // Draw background rectangle
        Rectangle::new(bg_color)
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
    font_size: u32,
    bg_color: [f32; 4],
    bg_color_hover: [f32; 4],
    bg_color_focus: [f32; 4],
    text_color: [f32; 4],
    
    position: [f64; 2],
    size: [f64; 2],
    
    pub has_focus: bool,
    
    mouse_focus: MouseFocus,
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
            
            mouse_focus: MouseFocus::NoHover,
        }
    }
    
    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use graphics::*;
        use graphics::text::Text;
        
        let bg_color =
            match self.mouse_focus {
                MouseFocus::NoHover => self.bg_color,
                MouseFocus::Hover => self.bg_color_hover,
                MouseFocus::Focus => self.bg_color_focus,
            };
    
        // Draw background rectangle
        Rectangle::new(bg_color)
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
            }
        });
        e.text(|text| {
            if self.has_focus {
                self.text.push_str(text);
            }
        });
    }
}