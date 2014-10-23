use rsfml::window::{keyboard, mouse, event};
use rsfml::graphics::{Color, Font, RenderTarget, RenderTexture, RenderWindow, Text, Texture};

use assets::LASER_TEXTURE;
use asset_store::AssetStore;
use battle_state::BattleContext;
use module::{MODULE_CATEGORIES, ModuleCategory};
use net::ClientId;
use sfml_renderer::SfmlRenderer;
use ship::{Ship, ShipRef};
use sim::SimVisuals;
use vec::{Vec2, Vec2f};

pub struct SpaceGui<'a> {
    module_category: Option<ModuleCategory>, // Selected module category
    
    // The target ships' render areas
    render_areas: Vec<ShipRenderArea>,
    
    // GUI font
    font: &'a Font,
    
    // Timer text
    timer_text: Text<'a>,
}

impl<'a> SpaceGui<'a> {
    pub fn new(font: &'a Font, my_client_id: ClientId, context: &BattleContext) -> SpaceGui<'a> {
        let mut render_areas = vec!();
        for (client_id, ship) in context.ships.iter() {
            if *client_id != my_client_id {
                let target = RenderTexture::new(500, 500, false).expect("Failed to create render texture");
                let texture = target.get_texture().expect("Failed to get render texture's texture");
                render_areas.push(ShipRenderArea {
                    ship: Some(ship.clone()),
                    position: Vec2{x: 772.0, y: 8.0},
                    target: target,
                    texture: texture,
                });
                break;
            }
        }
    
        SpaceGui {
            module_category: None,
            render_areas: render_areas,
            font: font,
            timer_text: Text::new_init("0", font, 30).expect("Failed to create timer text"),
        }
    }
    
    pub fn update(&mut self, window: &mut RenderWindow) {
        loop {
            match window.poll_event() {
                event::Closed => window.close(),
                event::KeyPressed{code, ..} => match code {
                    keyboard::Escape => { window.close(); },
                    code => { self.on_key_pressed(code); },
                },
                event::KeyReleased{..} => {},
                event::MouseButtonPressed{button, x, y} => {
                    match button {
                        mouse::MouseLeft => self.on_mouse_left_pressed(x, y),
                        _ => {},
                    }
                }
                /*event::MouseButtonReleased{button, x, y} => {
                }*/
                event::NoEvent => break,
                _ => {}
            };
        }
    }
    
    pub fn draw_planning(&mut self, renderer: &SfmlRenderer, asset_store: &AssetStore, client_ship: &Ship) {
        for render_area in self.render_areas.iter_mut() {
            (&mut render_area.target as &mut RenderTarget).clear(&Color::new_RGBA(255, 120, 0, 100));
            
            {
                let ship_renderer = SfmlRenderer::new(&render_area.target, asset_store);
                
                render_area.ship.as_ref().unwrap().borrow().draw(&ship_renderer);
            }
            
            render_area.target.display();
            renderer.draw_sf_texture_vec(&render_area.texture, &render_area.position);
        }
    
        self.draw_overlay(renderer, client_ship);
        
        self.timer_text.set_position2f(600.0, 8.0);
        self.timer_text.set_string("Planning");
        renderer.draw_text(&self.timer_text);
    }
    
    pub fn draw_simulating(&mut self, renderer: &SfmlRenderer, asset_store: &AssetStore, client_ship: &Ship, sim_visuals: &mut SimVisuals, time: f32) {
        for render_area in self.render_areas.iter_mut() {
            (&mut render_area.target as &mut RenderTarget).clear(&Color::new_RGBA(255, 120, 0, 100));
            
            {
                let ship_renderer = SfmlRenderer::new(&render_area.target, asset_store);
                
                render_area.ship.as_ref().unwrap().borrow().draw(&ship_renderer);
                sim_visuals.draw(&ship_renderer, render_area.ship.as_ref().unwrap().borrow().id, time);
            }
            
            render_area.target.display();
            renderer.draw_sf_texture_vec(&render_area.texture, &render_area.position);
        }
    
        self.draw_overlay(renderer, client_ship);
        
        self.timer_text.set_position2f(600.0, 8.0);
        self.timer_text.set_string("Simulating");
        renderer.draw_text(&self.timer_text);
    }
    
    fn draw_overlay(&self, renderer: &SfmlRenderer, client_ship: &Ship) {
        for category in MODULE_CATEGORIES.iter() {
            let icon_y: f32 =
                match self.module_category {
                    Some(c) if c == category.id => 584.0,
                    _ => { 600.0 },
                };
            
            renderer.draw_texture(LASER_TEXTURE, 10.0 + (64.0*(category.id as u8 as f32)), icon_y);
        }
        
        match self.module_category {
            Some(category) => {
                let mut i = 0u8;
                for module in client_ship.modules.iter() {
                    if module.borrow().get_base().category == category {                    
                        renderer.draw_texture(LASER_TEXTURE, 10.0 + (64.0*(i as f32)), 500.0);
                        i += 1;
                    }
                }
            },
            None => {},
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    fn on_mouse_left_pressed(&mut self, x: i32, y: i32) {
        for category in MODULE_CATEGORIES.iter() {
            let icon_x = 10 + (64*(category.id as i32));
            let icon_y: i32 =
                match self.module_category {
                    Some(c) if c == category.id => 584,
                    _ => { 600 },
                };
            let icon_w = 48;
            let icon_h = 48;
            
            if x >= icon_x && x <= icon_x+icon_w && y >= icon_y && y <= icon_y+icon_h {
                match self.module_category {
                    // Reclicked selected module category: deselect it
                    Some(c) if c == category.id => self.module_category = None,
                    // Selected a new module category
                    _ => self.module_category = Some(category.id),
                }
                return;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ShipRenderArea {
    ship: Option<ShipRef>,
    position: Vec2f,
    target: RenderTexture,
    texture: Texture,
}