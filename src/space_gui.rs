use std::rc::Rc;

use event::{Events, GenericEvent, RenderArgs};
use graphics::Context;
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use piston::Window;

use assets::GUI_TEXTURE;
use asset_store::AssetStore;
use battle_state::BattleContext;
use module::{IModule, ModuleRef};
use net::ClientId;
use ship::{Ship, ShipRef};
use sim::SimVisuals;
use vec::{Vec2, Vec2f};

pub struct SpaceGui<'a> {
    // The target ships' render areas
    render_areas: Vec<ShipRenderArea>,
    
    // Selected module
    module: Option<ModuleRef>,
    
    mouse_x: f64,
    mouse_y: f64,
    
    // Textures
    plan_texture: Texture,
    simulate_texture: Texture,
    win_texture: Texture,
    lose_texture: Texture,
}

impl<'a> SpaceGui<'a> {
    pub fn new(asset_store: &AssetStore, context: &BattleContext, my_client_id: ClientId) -> SpaceGui<'a> {
        let mut render_areas = vec!();
        for (i, (client_id, ship)) in context.ships_client_id.iter().filter(|s| *s.0 != my_client_id).enumerate() {
            if i < 2 {
                //let target = RenderTexture::new(500, 500, false).expect("Failed to create render texture");
                //let texture = target.get_texture().expect("Failed to get render texture's texture");
                render_areas.push(ShipRenderArea {
                    ship: Some(ship.clone()),
                    x: 772.0,
                    y: 8.0+(300.0 * (i as f64)),
                    //target: target,
                    //texture: texture,
                });
            }
        }
    
        SpaceGui {
            render_areas: render_areas,
            module: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            
            plan_texture: Texture::from_path(&Path::new("content/textures/gui/planning.png")).unwrap(),
            simulate_texture: Texture::from_path(&Path::new("content/textures/gui/simulating.png")).unwrap(),
            win_texture: Texture::from_path(&Path::new("content/textures/gui/win.png")).unwrap(),
            lose_texture: Texture::from_path(&Path::new("content/textures/gui/lose.png")).unwrap(),
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, client_ship: &mut Ship) {
        use event::*;
        
        if client_ship.state.get_hp() == 0 {
            return;
        }
    
        e.mouse_cursor(|x, y| {
            self.mouse_x = x;
            self.mouse_y = y;
        });
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                Button::Mouse(button) => {
                    let (mouse_x, mouse_y) = (self.mouse_x, self.mouse_y);
                    match button {
                        mouse::MouseButton::Left => self.on_mouse_left_pressed(mouse_x, mouse_y, client_ship),
                        mouse::MouseButton::Right => self.on_mouse_right_pressed(mouse_x, mouse_y, client_ship),
                        _ => {},
                    }
                },
            }
        });
    }
    
    pub fn reset_plan_stats(&mut self, client_ship: &mut Ship) {
        client_ship.state.plan_power = client_ship.state.power;
    }
    
    pub fn draw_planning(&mut self, context: &Context, gl: &mut Gl, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &Ship, time: f64) {
        use graphics::*;
        
        // Clear the screen
        clear([0.0, ..4], gl);
        
        // Draw player ship
        draw_ship(&context.trans(150.0, 150.0), gl, sim_visuals, client_ship, time);
    
        let mut enemy_alive = false;
        for render_area in self.render_areas.iter_mut() {
            // TODO clear render texture
        
            {
                let context = context.trans(render_area.x, render_area.y).trans(150.0, 150.0);
                
                draw_ship(&context, gl, sim_visuals, render_area.ship.as_ref().unwrap().borrow().deref(), time);
            }
            
            // TODO draw render texture
        
            if render_area.ship.as_ref().unwrap().borrow().state.get_hp() > 0 {
                enemy_alive = true;
            }
        }
        
        image(&self.plan_texture, &context.trans(550.0, 10.0), gl);
        
        if client_ship.state.get_hp() == 0 {
            image(&self.lose_texture, &context.trans(550.0, 100.0), gl);
        } else if !enemy_alive {
            image(&self.win_texture, &context.trans(550.0, 100.0), gl);
        }
    }
    
    pub fn draw_simulating(&mut self, context: &Context, gl: &mut Gl, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &Ship, time: f64) {
        use graphics::*;
        
        // Clear the screen
        clear([0.0, ..4], gl);
        
        // Draw player ship
        draw_ship(&context.trans(150.0, 150.0), gl, sim_visuals, client_ship, time);
    
        let mut enemy_alive = false;
        for render_area in self.render_areas.iter_mut() {
            // TODO clear render texture
        
            {
                let context = context.trans(render_area.x, render_area.y).trans(150.0, 150.0);
                
                draw_ship(&context, gl, sim_visuals, render_area.ship.as_ref().unwrap().borrow().deref(), time);
            }
            
            // TODO draw render texture
        
            if render_area.ship.as_ref().unwrap().borrow().state.get_hp() > 0 {
                enemy_alive = true;
            }
        }
        
        image(&self.simulate_texture, &context.trans(550.0, 10.0), gl);
        
        if client_ship.state.get_hp() == 0 {
            image(&self.lose_texture, &context.trans(550.0, 100.0), gl);
        } else if !enemy_alive {
            image(&self.win_texture, &context.trans(550.0, 100.0), gl);
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    fn on_mouse_left_pressed(&mut self, x: f64, y: f64, client_ship: &mut Ship) {
        if self.module.is_none() {
            let ship_offset_x = 150.0;
            let ship_offset_y = 150.0;
            let x = x - ship_offset_x;
            let y = y - ship_offset_y;

            for module in client_ship.modules.iter() {
                let mut module_borrowed = module.borrow_mut();
            
                // Get module position and size on screen
                let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
                let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
                let module_x = module_x as f64;
                let module_y = module_y as f64;
                let module_w = module_w as f64;
                let module_h = module_h as f64;
                if x >= module_x && x <= module_x+module_w && y >= module_y && y <= module_y+module_h {
                    if module_borrowed.get_base().plan_powered {
                        self.module = Some(module.clone());
                    } else if client_ship.state.can_activate_module(module_borrowed.get_base()) {
                        client_ship.state.activate_module(module_borrowed.get_base_mut());
                    }
                }
            }
        }
        
        for render_area in self.render_areas.iter() {
            let ship_offset_x = 150.0;
            let ship_offset_y = 150.0;
            let x = x - render_area.x - ship_offset_x;
            let y = y - render_area.y - ship_offset_y;
            match render_area.ship.as_ref() {
                Some(ship) => {
                    for module in ship.borrow().modules.iter() {
                        // Get module position and size on screen
                        let Vec2{x: module_x, y: module_y} = module.borrow().get_base().get_render_position();
                        let Vec2{x: module_w, y: module_h} = module.borrow().get_base().get_render_size();
                        let module_x = module_x as f64;
                        let module_y = module_y as f64;
                        let module_w = module_w as f64;
                        let module_h = module_h as f64;
                        if x >= module_x && x <= module_x+module_w && y >= module_y && y <= module_y+module_h {
                            if self.module.is_some() {
                                {
                                    // Inner scope for module ref so we can clear it after
                                    let selected_module = self.module.as_ref().unwrap();
                                    selected_module.borrow_mut().on_module_clicked(ship, module);
                                }
                                self.module = None;
                                return;
                            }
                        }
                    }
                },
                None => {},
            }
        }
    }
    
    fn on_mouse_right_pressed(&mut self, x: f64, y: f64, client_ship: &mut Ship) {
        if self.module.is_none() {
            let ship_offset_x = 150.0;
            let ship_offset_y = 150.0;
            let x = x - ship_offset_x;
            let y = y - ship_offset_y;

            for module in client_ship.modules.iter() {
                let mut module_borrowed = module.borrow_mut();
            
                // Get module position and size on screen
                let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
                let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
                let module_x = module_x as f64;
                let module_y = module_y as f64;
                let module_w = module_w as f64;
                let module_h = module_h as f64;
                if x >= module_x && x <= module_x+module_w && y >= module_y && y <= module_y+module_h {
                    let module_power = module_borrowed.get_base().get_power();
                    if module_borrowed.get_base().plan_powered {
                        client_ship.state.plan_power += module_power;
                        module_borrowed.get_base_mut().plan_powered = false;
                    }
                    return;
                }
            }
        }
        
        self.module = None;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ShipRenderArea {
    ship: Option<ShipRef>,
    x: f64,
    y: f64,
    //target: RenderTexture,
    //texture: Texture,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn draw_ship(context: &Context, gl: &mut Gl, sim_visuals: &mut SimVisuals, ship: &Ship, time: f64) {
    use current::Set;
    use graphics::*;

    sim_visuals.draw(context, gl, ship.id, time);
    ship.draw_module_hp(context, gl);
    
    let hp_rect = Rectangle::new([0.0, 1.0, 0.0, 1.0]);
    let shield_rect = Rectangle::new([0.0, 0.0, 1.0, 1.0]);
    let power_rect = Rectangle::new([1.0, 1.0, 0.0, 1.0]);
    let used_power_rect = Rectangle::new([1.0, 1.0, 0.0, 0.5]);
    
    for i in range(0, ship.state.get_hp()) {
        hp_rect.draw([(i as f64)*18.0, 0.0, 16.0, 32.0], &context.trans(-145.0, -145.0), gl);
    }
    
    for i in range(0, ship.state.shields) {
        shield_rect.draw([(i as f64)*18.0, 0.0, 16.0, 32.0], &context.trans(-145.0, -145.0 + 34.0), gl);
    }
    
    for i in range(0, ship.state.plan_power) {
        power_rect.draw([(i as f64)*18.0, 0.0, 16.0, 32.0], &context.trans(-145.0, -145.0 + 68.0), gl);
    }
    
    for i in range(ship.state.plan_power, ship.state.power) {
        used_power_rect.draw([(i as f64)*18.0, 0.0, 16.0, 32.0], &context.trans(-145.0, -145.0 + 68.0), gl);
    }
}