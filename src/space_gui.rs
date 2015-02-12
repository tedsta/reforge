use std::rand::Rng;
use std::rand;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};

use event::{Events, GenericEvent, RenderArgs};
use graphics::{Context, Rectangle};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use asset_store::AssetStore;
use battle_state::BattleContext;
use gui::TextButton;
use module;
use module::{IModule, ModuleBox, ModuleRef};
use net::ClientId;
use ship::{Ship, ShipId, ShipRef, ShipState};
use sim::SimVisuals;
use star_map_gui::{StarMapAction, StarMapGui};
use vec::{Vec2, Vec2f};

static SHIP_OFFSET_X: f64 = 80.0;
static SHIP_OFFSET_Y: f64 = 170.0;

static ENEMY_OFFSET_X: f64 = 80.0;
static ENEMY_OFFSET_Y: f64 = 50.0;

pub struct ModuleIcons {
    pub power_on_texture: Texture,
    pub power_off_texture: Texture,
}

pub struct StatsLabels {
    hp_texture: Texture,
    shield_texture: Texture,
    power_texture: Texture,
}

pub struct SpaceGui<'a> {
    // The target ships' render areas
    render_area: ShipRenderArea,
    
    // Selected module
    selection: Option<(ModuleRef, module::TargetMode)>,
    
    mouse_x: f64,
    mouse_y: f64,
    
    // Textures
    plan_texture: Texture,
    simulate_texture: Texture,
    win_texture: Texture,
    lose_texture: Texture,
    
    stats_labels: StatsLabels,
    
    module_icons: ModuleIcons,
    
    // Space background
    space_bg: SpaceStars,
    
    star_map_button: TextButton,
    star_map_gui: StarMapGui,
    show_star_map: bool,

    // targets
    target_icons: Vec<TargetIcon>,
}

impl<'a> SpaceGui<'a> {
    pub fn new(asset_store: &AssetStore, context: &BattleContext, my_ship_id: ShipId) -> SpaceGui<'a> {
        // Set up the render area
        //let target = RenderTexture::new(500, 500, false).expect("Failed to create render texture");
        //let texture = target.get_texture().expect("Failed to get render texture's texture");
        let x = 1280.0 - 5.0 - 560.0;
        let y = 128.0;
        let ship = context.ships_list.iter().filter(|ship| ship.borrow().id != my_ship_id).next().map(|ship| ship.clone());
        let render_area = ShipRenderArea {
            ship: ship,
            x: x,
            y: y,
            width: 560.0,
            height: (720.0 - 20.0) - y,
            //target: target,
            //texture: texture,
        };

        let target_icons = context.ships_list.iter().filter(|ship| ship.borrow().id != my_ship_id).take(5).map(|ship| TargetIcon { ship: ship.clone() }).collect();
    
        SpaceGui {
            render_area: render_area,
            selection: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            
            plan_texture: Texture::from_path(&Path::new("content/textures/gui/planning.png")).unwrap(),
            simulate_texture: Texture::from_path(&Path::new("content/textures/gui/simulating.png")).unwrap(),
            win_texture: Texture::from_path(&Path::new("content/textures/gui/win.png")).unwrap(),
            lose_texture: Texture::from_path(&Path::new("content/textures/gui/lose.png")).unwrap(),
            
            stats_labels: StatsLabels {
                hp_texture: Texture::from_path(&Path::new("content/textures/gui/hp_text.png")).unwrap(),
                shield_texture: Texture::from_path(&Path::new("content/textures/gui/shield_text.png")).unwrap(),
                power_texture: Texture::from_path(&Path::new("content/textures/gui/power_text.png")).unwrap(),
            },
            
            module_icons: ModuleIcons {
                power_on_texture: Texture::from_path(&Path::new("content/textures/gui/power_on_icon.png")).unwrap(),
                power_off_texture: Texture::from_path(&Path::new("content/textures/gui/power_off_icon.png")).unwrap(),
            },
            
            space_bg: SpaceStars::new(),
            
            star_map_button: TextButton::new("star map".to_string(), 20, [550.0, 50.0], [120.0, 40.0]),
            star_map_gui: StarMapGui::new(),
            show_star_map: false,

            target_icons: target_icons,
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, client_ship: &ShipRef) {
        use event::*;
        
        if client_ship.borrow().state.get_hp() == 0 {
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
        
        if let Some(star_map_result) = self.star_map_gui.event(e, [self.mouse_x - 200.0, self.mouse_y - 200.0]) {
            match star_map_result {
                StarMapAction::Close => {
                    self.show_star_map = false;
                },
            }
        }
        
        self.star_map_button.event(e, [self.mouse_x, self.mouse_y]);
        if self.star_map_button.get_clicked() {
            self.show_star_map = true;
        }
    }
    
    pub fn reset_plan_stats(&mut self, client_ship: &mut Ship) {
        client_ship.state.plan_power = client_ship.state.power;
    }
    
    pub fn draw_planning(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &mut Ship, time: f64, dt: f64) {
        use graphics::*;
        
        // Clear the screen
        clear([0.0; 4], gl);
        
        self.draw_screen(context, gl, glyph_cache, asset_store, sim_visuals, client_ship, time, dt);
        
        // Draw planning text
        image(&self.plan_texture, &context.trans(550.0, 10.0), gl);
        
        if self.show_star_map {
            self.star_map_gui.draw(&context.trans(200.0, 200.0), gl, glyph_cache);
        }
    }
    
    pub fn draw_simulating(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &mut Ship, time: f64, dt: f64) {
        use graphics::*;
        
        // Clear the screen
        clear([0.0; 4], gl);
        
        self.draw_screen(context, gl, glyph_cache, asset_store, sim_visuals, client_ship, time, dt);
        
        // Draw simulating text
        image(&self.simulate_texture, &context.trans(550.0, 10.0), gl);
        
        if self.show_star_map {
            self.star_map_gui.draw(&context.trans(200.0, 200.0), gl, glyph_cache);
        }
    }
    
    fn draw_screen(&mut self, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &mut Ship, time: f64, dt: f64) {
        use graphics::*;
        
        // Draw the space background
        self.space_bg.update(dt);
        self.space_bg.draw(context, gl);
        
        // Draw player ship
        draw_ship(&context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl, sim_visuals, client_ship, time);
        client_ship.draw_module_powered_icons(&context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl, &self.module_icons);
        draw_stats(context, gl, glyph_cache, &self.stats_labels, client_ship.deref());
    
        let mut enemy_alive = false;
        if let Some(ref ship) = self.render_area.ship {
            // TODO clear render texture
            
            Rectangle::new([1.0, 0.7, 0.2, 0.5]).draw([self.render_area.x, self.render_area.y, self.render_area.width, self.render_area.height], context, gl);
        
            {
                let context = context.trans(self.render_area.x, self.render_area.y);
                
                draw_ship(&context.trans(ENEMY_OFFSET_X, ENEMY_OFFSET_Y), gl, sim_visuals, ship.borrow().deref(), time);
                draw_stats(&context.trans(0.0, 375.0), gl, glyph_cache, &self.stats_labels, ship.borrow().deref());
            }
            
            // TODO draw render texture
        
            if ship.borrow().state.get_hp() > 0 {
                enemy_alive = true;
            }
        }
        
        // If not currently selecting a module, highlight modules the user mouses-over
        if self.selection.is_none() {
            let x = self.mouse_x - SHIP_OFFSET_X;
            let y = self.mouse_y - SHIP_OFFSET_Y;

            apply_to_module_if_point_inside(client_ship, x, y, |ship_state, _, module_borrowed| {
                let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
                let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
                let (module_x, module_y, module_w, module_h) = (module_x as f64, module_y as f64, module_w as f64, module_h as f64);
            
                if module_borrowed.get_base().plan_powered {
                    Rectangle::new([0.0, 0.0, 1.0, 0.5])
                        .draw([module_x, module_y, module_w, module_h], &context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl);
                } else if ship_state.can_plan_activate_module(module_borrowed.get_base()) {
                    Rectangle::new([1.0, 1.0, 0.0, 0.5])
                        .draw([module_x, module_y, module_w, module_h], &context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl);
                }
            });
        }
        
        if client_ship.state.get_hp() == 0 {
            image(&self.lose_texture, &context.trans(550.0, 100.0), gl);
        } else if !enemy_alive {
            image(&self.win_texture, &context.trans(550.0, 100.0), gl);
        }
        
        self.star_map_button.draw(context, gl, glyph_cache);

        // draw target icons
        for (i, icon) in self.target_icons.iter().enumerate() {
            let i = i as f64;

            if let Some(ref ship) = self.render_area.ship {
                if ship.borrow().id == icon.ship.borrow().id {
                    Rectangle::new([0.0, 0.0, 1.0, 1.0])
                        .draw([715.0+(i*100.0), 5.0, 96.0, 96.0], context, gl);
                } else {
                    Rectangle::new([1.0, 0.0, 0.0, 1.0])
                        .draw([715.0+(i*100.0), 5.0, 96.0, 96.0], context, gl);
                }
            } else {
                Rectangle::new([1.0, 0.0, 0.0, 1.0])
                    .draw([715.0+(i*100.0), 5.0, 96.0, 96.0], context, gl);
            }
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    fn on_mouse_left_pressed(&mut self, x: f64, y: f64, client_ship: &ShipRef) {
        if self.selection.is_none() {
            let x = x - SHIP_OFFSET_X;
            let y = y - SHIP_OFFSET_Y;
            
            apply_to_module_if_point_inside(client_ship.borrow_mut().deref_mut(), x, y, |ship_state, module, module_borrowed| {
                if module_borrowed.get_base().plan_powered {
                    if let Some(target_mode) = module_borrowed.get_target_mode() {
                        // Select this module and begin targeting
                        self.selection = Some((module.clone(), target_mode));
                    }
                } else if ship_state.can_plan_activate_module(module_borrowed.get_base()) {
                    ship_state.activate_module(module_borrowed.get_base_mut());
                }
            });
        }
        
        let mut clear_selection = false;
        
        if let Some(ref selection) = self.selection {
            let &(ref selected_module, ref target_mode) = selection;

            match *target_mode {
                module::TargetMode::TargetModule => {
                    let x = x - self.render_area.x - ENEMY_OFFSET_X;
                    let y = y - self.render_area.y - ENEMY_OFFSET_Y;
                    if let Some(ref ship) = self.render_area.ship {
                        apply_to_module_if_point_inside(ship.borrow_mut().deref_mut(), x, y, |_, module, _| {
                            selected_module.borrow_mut().get_base_mut().target_data =
                                Some(module::TargetData::TargetModule(ship.clone(), module.clone()));
                            clear_selection = true;
                        });
                    }
                },
                module::TargetMode::OwnModule => {
                    let x = x - SHIP_OFFSET_X;
                    let y = y - SHIP_OFFSET_Y;
                    apply_to_module_if_point_inside(client_ship.borrow_mut().deref_mut(), x, y, |_, module, _| {
                        selected_module.borrow_mut().get_base_mut().target_data =
                            Some(module::TargetData::OwnModule(client_ship.clone(), module.clone()));
                        clear_selection = true;
                    });
                },
                _ => {},
            }
        }
        
        if clear_selection {
            self.selection = None;
        }

        for (i, icon) in self.target_icons.iter().enumerate() {
            let i = i as f64;
            let icon_x = 715.0+(i*100.0);
            let icon_y = 5.0;
            let icon_w = 96.0;
            let icon_h = 96.0;

            if x >= icon_x && x <= icon_x+icon_w && y >= icon_y && y <= icon_y+icon_h {
                let mut should_change = false;

                if let Some(ref ship) = self.render_area.ship { // switching to a new ship
                    if ship.borrow().id != icon.ship.borrow().id {
                        should_change = true;
                    } else {
                        // do nothing
                    }
                } 
                if should_change {
                    self.render_area.ship = Some(icon.ship.clone());
                    break;
                }
            }
        }
    }
    
    fn on_mouse_right_pressed(&mut self, x: f64, y: f64, client_ship: &ShipRef) {
        let mut module_was_deactivated = false;
    
        if self.selection.is_none() {
            let x = x - SHIP_OFFSET_X;
            let y = y - SHIP_OFFSET_Y;
            
            apply_to_module_if_point_inside(client_ship.borrow_mut().deref_mut(), x, y, |ship_state, module, module_borrowed| {
                let module_power = module_borrowed.get_base().get_power();
                if module_borrowed.get_base().plan_powered {
                    ship_state.deactivate_module(module_borrowed.get_base_mut());
                }
                module_was_deactivated = true;
            });
        }
        
        if !module_was_deactivated {
            self.selection = None;
        }
    }
    
    pub fn try_lock(&mut self, ship: &ShipRef) {
        if self.render_area.ship.is_none() {
            self.render_area.ship = Some(ship.clone());
        }
        
        if self.target_icons.len() < 5 {
            self.target_icons.push(TargetIcon { ship: ship.clone() });
        }
    }
    
    pub fn remove_lock(&mut self, ship_id: ShipId) {
        if self.render_area.ship.is_some() && self.render_area.ship.as_ref().unwrap().borrow().id == ship_id {
            self.render_area.ship = None;
        }
        
        self.target_icons.retain(|i| i.ship.borrow().id != ship_id);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Applies function to a module in the ship if the mouse is over the module.
/// Returns whether or not the function was applied.
pub fn apply_to_module_if_point_inside<F>(ship: &mut Ship, x: f64, y: f64, mut f: F)
    where
        F: FnMut(&mut ShipState, &ModuleRef, &mut ModuleBox)
{
    for module in ship.modules.iter() {
        let mut module_borrowed = module.borrow_mut();
    
        // Get module position and size on screen
        let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
        let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
        let (module_x, module_y, module_w, module_h) = (module_x as f64, module_y as f64, module_w as f64, module_h as f64);
        if x >= module_x && x <= module_x+module_w && y >= module_y && y <= module_y+module_h {
            f(&mut ship.state, module, module_borrowed.deref_mut());
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TargetIcon {
    ship: ShipRef,
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
    sim_visuals.draw(context, gl, ship.id, time);
    ship.draw_module_hp(context, gl);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn draw_stats(context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache, stats_labels: &StatsLabels, ship: &Ship) {
    use graphics::*;
    use graphics::text::Text;
    
    let hp_rect = Rectangle::new([0.0, 1.0, 0.0, 1.0]);
    let shield_rect = Rectangle::new([0.0, 0.0, 1.0, 1.0]);
    let power_rect = Rectangle::new([1.0, 1.0, 0.0, 1.0]);
    let used_power_rect = Rectangle::new([1.0, 1.0, 0.0, 0.5]);
    let new_power_rect = Rectangle::new([0.0, 1.0, 0.0, 0.5]);
    
    for i in range(0, ship.state.get_hp()) {
        hp_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.trans(5.0, 5.0 + 14.0), gl);
    }
    
    for i in range(0, ship.state.shields) {
        shield_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.trans(5.0, 5.0 + 52.0), gl);
    }
    
    for i in range(0, ship.state.plan_power) {
        power_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.trans(5.0, 5.0 + 90.0), gl);
    }
    
    if ship.state.plan_power < ship.state.power {
        for i in range(ship.state.plan_power, ship.state.power) {
            used_power_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.trans(5.0, 5.0 + 90.0), gl);
        }
    } else if ship.state.plan_power > ship.state.power {
        for i in range(ship.state.power, ship.state.plan_power) {
            new_power_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.trans(5.0, 5.0 + 90.0), gl);
        }
    }
    
    // Draw labels for hp, shields and power meters
    image(&stats_labels.hp_texture, &context.trans(5.0, 4.0), gl);
    image(&stats_labels.shield_texture, &context.trans(5.0, 42.0), gl);
    image(&stats_labels.power_texture, &context.trans(5.0, 80.0), gl);
    
    Text::colored([1.0; 4], 30).draw(
        ship.name.as_slice(),
        glyph_cache,
        &context.trans(5.0, 160.0),
        gl,
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Yo dawg imma draw me some space stars

struct Star {
    position: [f64; 2],
    size: f64,
}

struct SpaceStars {
    stars: Vec<Vec<Star>>, // Layers of stars
}

impl SpaceStars {
    pub fn new() -> SpaceStars {
        // Random number generater
        let mut rng = rand::thread_rng();
        
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
                star.position[0] -= (50.0/i)*dt;
                
                if star.position[0] < 0.0 - star.size {
                    star.position[0] += 1290.0
                }
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
