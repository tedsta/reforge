use piston::event::GenericEvent;
use graphics::Context;
use piston::input::{keyboard, mouse, Button};
use opengl_graphics::{GlGraphics, Texture};
use opengl_graphics::glyph_cache::GlyphCache;
use std::path::Path;

use gui::{TextBox, TextButton};
use vec::Vec2f;

use super::ChatMsg;

pub enum ChatGuiAction {
    SendMsg(String),
}

pub struct ChatGui {
    action: Option<ChatGuiAction>,
    
    msg_box: TextBox,
    chat_base: Texture,
    chat_slider: Texture,
    
    dragging: bool,
    
    messages: Vec<ChatMsg>,
}

impl ChatGui {
    pub fn new() -> ChatGui {
        let mut msg_box = TextBox::new("".to_string(), 10, [5.0, 175.0], [238.0, 20.0]);
        msg_box.bg_color = [0.0, 0.0, 0.0, 0.0];
        msg_box.bg_color_hover = [0.0, 0.0, 0.0, 0.0];
        msg_box.bg_color_focus = [0.0, 0.0, 0.0, 0.0];
        
        ChatGui {
            action: None,
            
            msg_box: msg_box,
            chat_base: Texture::from_path(&Path::new("content/textures/gui/textbase.png")).unwrap(),
            chat_slider: Texture::from_path(&Path::new("content/textures/gui/textglass.png")).unwrap(),
            
            dragging: false,
            
            messages: vec!(),
        }
    }
    
    pub fn add_message(&mut self, msg: ChatMsg) {
        self.messages.push(msg);
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: Vec2f) -> Option<ChatGuiAction> {
        use piston::event::*;
        
        self.msg_box.event(e, [mouse_pos.x, mouse_pos.y]);
        
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => self.on_mouse_left_pressed(mouse_pos.x, mouse_pos.y),
                        _ => {},
                    }
                },
            }
        });
        
        e.release(|button| {
            match button {
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => self.on_mouse_left_released(mouse_pos.x, mouse_pos.y),
                        _ => {},
                    }
                }, 
                _ => { },
            }
        });
        
        e.mouse_relative(|x, y| {
            self.on_mouse_moved(mouse_pos.x, mouse_pos.y, x, y);
        });
        
        self.action.take()
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
        match key {
            keyboard::Key::Return => {
                if self.msg_box.text.len() > 0 && self.msg_box.has_focus {
                    self.action = Some(ChatGuiAction::SendMsg(self.msg_box.text.clone()));
                    self.msg_box.text = "".to_string();
                }
            },
            _ => { },
        }
    }
    
    fn on_mouse_left_pressed(&mut self, x: f64, y: f64,) {
        self.dragging = true;
    }
    
    fn on_mouse_left_released(&mut self, x: f64, y: f64,) {
        self.dragging = false;
    }
    
    fn on_mouse_moved(&mut self, x: f64, y: f64, moveX: f64, moveY: f64) {
        println!("{}, {}", moveX, moveY);
    }

    pub fn draw(&mut self, context: &Context, gl: &mut GlGraphics, glyph_cache: &mut GlyphCache) {
        use graphics::*;
        use graphics::text::Text;
        
        image(&self.chat_slider, context.trans(0.0, 0.0).transform, gl);
        image(&self.chat_base, context.trans(0.0, 73.0).transform, gl);
        
        {
            let max_messages = 10;
            for(i, msg) in self.messages.iter().rev().take(max_messages).enumerate() {
                let context = context.trans(0.0, 18.0 + 15.0*((max_messages - 1 - i) as f64));
                Text::colored([0.7, 0.7, 1.0, 1.0], 10).draw(
                    msg.author_name.as_str(),
                    glyph_cache,
                    &context.draw_state, context.transform,
                    gl,
                );
                
                let context = context.trans(msg.author_name.len() as f64 * 10.0, 0.0);
                Text::colored([0.7, 0.7, 0.7, 1.0], 10).draw(
                    msg.content.as_str(),
                    glyph_cache,
                    &context.draw_state, context.transform,
                    gl,
                );
            }
        }
        
        self.msg_box.draw(context, gl, glyph_cache);
    }
}
