use event::GenericEvent;
use graphics::Context;
use input::{keyboard, Button};
use opengl_graphics::Gl;
use opengl_graphics::glyph_cache::GlyphCache;

use gui::{TextBox, TextButton};
use vec::Vec2f;

use super::ChatMsg;

pub enum ChatGuiAction {
    SendMsg(String),
}

pub struct ChatGui {
    action: Option<ChatGuiAction>,
    
    msg_box: TextBox,
    send_button: TextButton,
    
    messages: Vec<ChatMsg>,
}

impl ChatGui {
    pub fn new() -> ChatGui {
        ChatGui {
            action: None,
            
            msg_box: TextBox::new("".to_string(), 14, [5.0, 175.0], [238.0, 20.0]),
            send_button: TextButton::new("send".to_string(), 14, [245.0, 175.0], [50.0, 20.0]),
            
            messages: vec!(),
        }
    }
    
    pub fn add_message(&mut self, msg: ChatMsg) {
        self.messages.push(msg);
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: Vec2f) -> Option<ChatGuiAction> {
        use event::*;
        
        self.msg_box.event(e, [mouse_pos.x, mouse_pos.y]);
        self.send_button.event(e, [mouse_pos.x, mouse_pos.y]);
        
        if self.send_button.get_clicked() {
            if self.msg_box.text.len() > 0 {
                self.action = Some(ChatGuiAction::SendMsg(self.msg_box.text.clone()));
                self.msg_box.text = "".to_string();
            }
        }
        
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                _ => { },
            }
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

    pub fn draw(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache) {
        use quack::Set;
        use graphics::*;
        use graphics::text::Text;
        
        // Render background window
        Rectangle::new([0.2, 0.05, 0.3, 0.8])
            .draw([0.0, 0.0, 300.0, 200.0], &context.draw_state, context.transform, gl);
        
        {
            // Label text
            let context = context.trans(5.0, 20.0);
            Text::colored([1.0; 4], 15).draw(
                "chat",
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
            
            let max_messages = 10;
            for(i, msg) in self.messages.iter().rev().take(max_messages).enumerate() {
                let context = context.trans(0.0, 16.0 + 15.0*((max_messages - 1 - i) as f64));
                Text::colored([0.7, 0.7, 1.0, 1.0], 14).draw(
                    msg.author_name.as_slice(),
                    glyph_cache,
                    &context.draw_state, context.transform,
                    gl,
                );
                
                let context = context.trans(msg.author_name.len() as f64 * 14.0, 0.0);
                Text::colored([0.7, 0.7, 0.7, 1.0], 14).draw(
                    msg.content.as_slice(),
                    glyph_cache,
                    &context.draw_state, context.transform,
                    gl,
                );
            }
        }
        
        self.msg_box.draw(context, gl, glyph_cache);
        self.send_button.draw(context, gl, glyph_cache);
    }
}