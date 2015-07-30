use std::rc::Rc;

use piston::event::GenericEvent;
use graphics::Context;
use piston::input::{keyboard, mouse, Button};
use opengl_graphics::{GlGraphics, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use asset_store::AssetStore;
use battle_context::BattleContext;
use gui::TextButton;
use ship::{Ship, ShipIndex};
use vec::{Vec2, Vec2f};

pub enum NavMapGuiAction {
    Close,
}

pub struct NavMapGui {
    scale: f64,
    move_dir: Vec2f,

    action: Option<NavMapGuiAction>,
    
    // Buttons
    close_button: TextButton,
    
    frame: Rc<Texture>,
}

impl NavMapGui {
    pub fn new(asset_store: &AssetStore) -> NavMapGui {
        NavMapGui {
            scale: 1.0,
            move_dir: Vec2::new(0.0, 0.0),
        
            action: None,
            
            close_button: TextButton::new("Close".to_string(), 20, [450.0, 400.0], [150.0, 40.0]),
            
            frame: asset_store.get_texture_str("nav_map").clone(),
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, mouse_pos: [f64; 2], bc: &mut BattleContext,
                                  client_ship: ShipIndex) -> Option<NavMapGuiAction> {
        use piston::event::*;
        
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key),
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => {
                            self.on_mouse_left_pressed(mouse_pos, button, bc, client_ship);
                        },
                        mouse::MouseButton::Right => { },
                        _ => {},
                    }
                },
            }
        });
        
        // Handle buttons
        self.close_button.event(e, mouse_pos);
        
        if self.close_button.get_clicked() {
            self.action = Some(NavMapGuiAction::Close);
        }
        
        self.action.take()
    }

    fn on_mouse_left_pressed(&mut self, mouse_pos: [f64; 2], button: mouse::MouseButton, bc: &BattleContext, client_ship: ShipIndex) {
        let mouse_pos = Vec2::new(mouse_pos[0] - 288.0, mouse_pos[1] - 202.0);
        let radar_center = client_ship.get(bc).position;
    
        // If inside circle clicked
        if mouse_pos.length() < 160.0 {
            // Check if space object was selected
            for ship in bc.ships_iter() {
                // Check that ship's icon if it's in the radar
                let mut screen_pos = (ship.position - radar_center) * self.scale;
                let ship_radius = f64::max(ship.get_width() as f64, ship.get_height() as f64);
                
                if screen_pos.length() < 160.0 + ship_radius {
                    screen_pos.y *= -1.0;
                    screen_pos = screen_pos*self.scale;
                    let size = Vec2::new(ship.get_width() as f64, ship.get_height() as f64);
                    let half_size = size / 2.0;
                    if mouse_pos.x > screen_pos.x-half_size.x && mouse_pos.x < screen_pos.x+half_size.x &&
                       mouse_pos.y > screen_pos.y-half_size.y && mouse_pos.y < screen_pos.y+half_size.y {
                        // TODO select the ship
                    }
                }
            }

            // If nothing was selected, then it's a waypoint
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }

    pub fn draw(&mut self, context: &Context, gl: &mut GlGraphics, glyph_cache: &mut GlyphCache,
                bc: &BattleContext, client_ship: &Ship) {
        use graphics::*;
        
        Ellipse::new([0.0, 0.5, 0.0, 1.0])
                .draw([118.0, 32.0, 340.0, 340.0], &context.draw_state, context.transform, gl);
        
        // Render all the stuff in the nav map
        {
            let context = context.trans(288.0, 202.0);
            
            for ship in bc.ships_iter() {
                // Draw ship's icon if it's in the radar
                let screen_pos = ship.position * self.scale;
                
                if screen_pos.length() < 170.0 {
                    let context = context.trans(screen_pos.x, -screen_pos.y).scale(self.scale, self.scale);
                    let size = Vec2::new(ship.get_width() as f64, ship.get_height() as f64);
                    let half_size = size / 2.0;
                    let color =
                        if ship.index == client_ship.index {
                            [0.0, 1.0, 0.0, 1.0]
                        } else {
                            [1.0, 0.0, 0.0, 1.0]
                        };
                    Rectangle::new(color)
                        .draw([-half_size.x, -half_size.y, size.x, size.y],
                               &context.draw_state, context.transform, gl);
                }
            }
        }
        
        image(&*self.frame, context.transform, gl);
        
        // Draw the buttons
        self.close_button.draw(context, gl, glyph_cache);
    }
}

fn lerp_ship_waypoint(ship: &mut Ship, time: f64) -> Vec2f {
    /*if ship.waypoints.len() > 0 {
        let next_pos = ship.waypoints[0];
        ship.position + next_pos*(time/5.0)
    } else {
        ship.position
    }*/
    Vec2::new(0.0, 0.0)
}
