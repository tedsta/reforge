use std::rand::Rng;
use std::rand;
use std::rc::Rc;

use event::{Events, GenericEvent, RenderArgs};
use graphics::{Context, Rectangle};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};

use assets::GUI_TEXTURE;
use asset_store::AssetStore;
use battle_state::BattleContext;
use module::{IModule, ModuleRef};
use net::ClientId;
use ship::{Ship, ShipId, ShipRef};
use sim::SimVisuals;
use vec::{Vec2, Vec2f};

static SHIP_OFFSET_X: f64 = 100.0;
static SHIP_OFFSET_Y: f64 = 170.0;

pub struct ModuleIcons {
    pub power_on_texture: Texture,
    pub power_off_texture: Texture,
}

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
    hp_texture: Texture,
    shield_texture: Texture,
    power_texture: Texture,
    
    module_icons: ModuleIcons,
    
    // Space background
    space_bg: SpaceStars,
}

impl<'a> SpaceGui<'a> {
    pub fn new(asset_store: &AssetStore, context: &BattleContext, my_client_id: ClientId) -> SpaceGui<'a> {
        let mut render_areas = vec!();
        for (i, ship) in context.ships_list.iter().filter(|ship| ship.borrow().client_id != Some(my_client_id)).enumerate() {
            if i < 2 {
                //let target = RenderTexture::new(500, 500, false).expect("Failed to create render texture");
                //let texture = target.get_texture().expect("Failed to get render texture's texture");
                let x = 700.0;
                let y = (360.0 * (i as f64));
                render_areas.push(ShipRenderArea {
                    ship: Some(ship.clone()),
                    x: x,
                    y: y,
                    width: 1280.0 - x,
                    height: 360.0,
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
            hp_texture: Texture::from_path(&Path::new("content/textures/gui/hp_text.png")).unwrap(),
            shield_texture: Texture::from_path(&Path::new("content/textures/gui/shield_text.png")).unwrap(),
            power_texture: Texture::from_path(&Path::new("content/textures/gui/power_text.png")).unwrap(),
            
            module_icons: ModuleIcons {
                power_on_texture: Texture::from_path(&Path::new("content/textures/gui/power_on_icon.png")).unwrap(),
                power_off_texture: Texture::from_path(&Path::new("content/textures/gui/power_off_icon.png")).unwrap(),
            },
            
            space_bg: SpaceStars::new(),
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
    
    pub fn draw_planning(&mut self, context: &Context, gl: &mut Gl, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &Ship, time: f64, dt: f64) {
        use graphics::*;
        
        // Clear the screen
        clear([0.0, ..4], gl);
        
        // Draw the space background
        self.space_bg.update(dt);
        self.space_bg.draw(context, gl);
        
        // Draw player ship
        draw_ship(&context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl, sim_visuals, client_ship, time);
        client_ship.draw_module_powered_icons(&context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl, &self.module_icons);
    
        let mut enemy_alive = false;
        for render_area in self.render_areas.iter_mut() {
            if let Some(ref ship) = render_area.ship {
                // TODO clear render texture
                
                Rectangle::new([1.0, 0.7, 0.2, 0.5]).draw([render_area.x, render_area.y, render_area.width, render_area.height], context, gl);
            
                {
                    let context = context.trans(render_area.x, render_area.y).trans(SHIP_OFFSET_X, SHIP_OFFSET_Y);
                    
                    draw_ship(&context, gl, sim_visuals, ship.borrow().deref(), time);
                }
                
                // TODO draw render texture
            
                if ship.borrow().state.get_hp() > 0 {
                    enemy_alive = true;
                }
            }
        }
        
        image(&self.plan_texture, &context.trans(550.0, 10.0), gl);

        // Draw labels for hp, shields and power meters
        image(&self.hp_texture, &context.trans(5.0, 4.0), gl);
        image(&self.shield_texture, &context.trans(5.0, 58.0), gl);
        image(&self.power_texture, &context.trans(5.0, 110.0), gl);
        
        if self.module.is_none() {
            let x = self.mouse_x - SHIP_OFFSET_X;
            let y = self.mouse_y - SHIP_OFFSET_Y;

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
                        Rectangle::new([0.0, 0.0, 1.0, 0.5])
                            .draw([module_x, module_y, module_w, module_h], &context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl);
                    } else if client_ship.state.can_activate_module(module_borrowed.get_base()) {
                        Rectangle::new([1.0, 1.0, 0.0, 0.5])
                            .draw([module_x, module_y, module_w, module_h], &context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl);
                    }
                }
            }
        }
        
        if client_ship.state.get_hp() == 0 {
            image(&self.lose_texture, &context.trans(550.0, 100.0), gl);
        } else if !enemy_alive {
            image(&self.win_texture, &context.trans(550.0, 100.0), gl);
        }
    }
    
    pub fn draw_simulating(&mut self, context: &Context, gl: &mut Gl, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &Ship, time: f64, dt: f64) {
        use graphics::*;
        
        // Clear the screen
        clear([0.0, ..4], gl);
        
        // Draw the space background
        self.space_bg.update(dt);
        self.space_bg.draw(context, gl);
        
        // Draw player ship
        draw_ship(&context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl, sim_visuals, client_ship, time);
        client_ship.draw_module_powered_icons(&context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl, &self.module_icons);
    
        let mut enemy_alive = false;
        for render_area in self.render_areas.iter_mut() {
            if let Some(ref ship) = render_area.ship {
                // TODO clear render texture
                
                Rectangle::new([1.0, 0.7, 0.2, 0.5]).draw([render_area.x, render_area.y, render_area.width, render_area.height], context, gl);
            
                {
                    let context = context.trans(render_area.x, render_area.y).trans(SHIP_OFFSET_X, SHIP_OFFSET_Y);
                    
                    draw_ship(&context, gl, sim_visuals, ship.borrow().deref(), time);
                }
                
                // TODO draw render texture
            
                if ship.borrow().state.get_hp() > 0 {
                    enemy_alive = true;
                }
            }
        }
        
        image(&self.simulate_texture, &context.trans(550.0, 10.0), gl);
        
        // Draw labels for hp, shields and power meters
        image(&self.hp_texture, &context.trans(5.0, 4.0), gl);
        image(&self.shield_texture, &context.trans(5.0, 58.0), gl);
        image(&self.power_texture, &context.trans(5.0, 110.0), gl);
        
        if self.module.is_none() {
            let x = self.mouse_x - SHIP_OFFSET_X;
            let y = self.mouse_y - SHIP_OFFSET_Y;

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
                        Rectangle::new([0.0, 0.0, 1.0, 0.5])
                            .draw([module_x, module_y, module_w, module_h], &context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl);
                    } else if client_ship.state.can_activate_module(module_borrowed.get_base()) {
                        Rectangle::new([1.0, 1.0, 0.0, 0.5])
                            .draw([module_x, module_y, module_w, module_h], &context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl);
                    }
                }
            }
        }
        
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
            let x = x - SHIP_OFFSET_X;
            let y = y - SHIP_OFFSET_Y;

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
            let x = x - render_area.x - SHIP_OFFSET_X;
            let y = y - render_area.y - SHIP_OFFSET_Y;
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
            let x = x - SHIP_OFFSET_X;
            let y = y - SHIP_OFFSET_Y;

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
                        client_ship.state.deactivate_module(module_borrowed.get_base_mut());
                    }
                    return;
                }
            }
        }
        
        self.module = None;
    }
    
    pub fn try_lock(&mut self, ship: &ShipRef) {
        for render_area in self.render_areas.iter_mut() {
            if render_area.ship.is_none() {
                render_area.ship = Some(ship.clone());
                break;
            }
        }
    }
    
    pub fn remove_lock(&mut self, ship_id: ShipId) {
        for render_area in self.render_areas.iter_mut() {
            if render_area.ship.is_some() && render_area.ship.as_ref().unwrap().borrow().id == ship_id {
                render_area.ship = None;
                break;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ShipRenderArea {
    ship: Option<ShipRef>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
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
        hp_rect.draw([(i as f64)*18.0, 0.0, 16.0, 32.0], &context.trans(-(SHIP_OFFSET_X - 5.0), -(SHIP_OFFSET_Y - 5.0) + 14.0), gl);
    }
    
    for i in range(0, ship.state.shields) {
        shield_rect.draw([(i as f64)*18.0, 0.0, 16.0, 32.0], &context.trans(-(SHIP_OFFSET_X - 5.0), -(SHIP_OFFSET_Y - 5.0) + 68.0), gl);
    }
    
    for i in range(0, ship.state.plan_power) {
        power_rect.draw([(i as f64)*18.0, 0.0, 16.0, 32.0], &context.trans(-(SHIP_OFFSET_X - 5.0), -(SHIP_OFFSET_Y - 5.0) + 120.0), gl);
    }
    
    for i in range(ship.state.plan_power, ship.state.power) {
        used_power_rect.draw([(i as f64)*18.0, 0.0, 16.0, 32.0], &context.trans(-(SHIP_OFFSET_X - 5.0), -(SHIP_OFFSET_Y - 5.0) + 120.0), gl);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Yo dawk imma draw me some space stars

struct Star {
    position: [f64, .. 2],
    size: f64,
}

struct SpaceStars {
    stars: Vec<Vec<Star>>, // Layers of stars
}

impl SpaceStars {
    pub fn new() -> SpaceStars {
        // Random number generater
        let mut rng = rand::task_rng();
        
        // Generate a bunch of stars
        let mut stars = Vec::with_capacity(5); // Five layers of stars
        for _ in range(0, stars.capacity()) {
            let mut layer = Vec::with_capacity(50);
            for _ in range(0u8, 20) {
                layer.push(Star {
                    position: [rng.gen::<f64>() * 1290.0, rng.gen::<f64>() * 730.0],
                    size: (rng.gen::<u8>() % 5 + 1) as f64,
                });
            }
            stars.push(layer);
        }
    
        SpaceStars {
            stars: stars,
        }
    }
    
    pub fn update(&mut self, dt: f64) {
        for (i, stars) in self.stars.iter_mut().enumerate() {
            let i = i as f64;
            for star in stars.iter_mut() {
                star.position[0] += 20.0*dt / i;
                //star.position[1] += 10.0*dt / i;
                
                if star.position[0] > 1280.0 {
                    star.position[0] -= 1290.0
                }
                /*if star.position[1] > 720.0 {
                    star.position[1] -= 730.0
                }*/
            }
        }
    }
    
    pub fn draw(&self, context: &Context, gl: &mut Gl) {
        use graphics::*;
        
        let star_rect = Rectangle::new([1.0, 1.0, 1.0, 1.0]);
        for stars in self.stars.iter() {
            for star in stars.iter() {
                star_rect.draw([star.position[0], star.position[1], star.size, star.size], context, gl);
            }
        }
    }
}
