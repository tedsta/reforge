use std::rand::Rng;
use std::rand;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use event::{Events, GenericEvent, RenderArgs};
use graphics::{Context, Rectangle};
use input::{keyboard, mouse, Button};
use opengl_graphics::{Gl, Texture};
use opengl_graphics::glyph_cache::GlyphCache;

use asset_store::AssetStore;
use battle_context::BattleContext;
use gui::TextButton;
use module;
use module::{IModule, ModuleBox, ModuleRef};
use net::ClientId;
use sector_data::SectorData;
use ship::{Ship, ShipId, ShipIndex, ShipPlans, ShipRef, ShipState};
use sim::SimEffects;
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

pub struct SpaceGui {
    // Plans for player's ship
    plans: ShipPlans,

    // The target ships' render areas
    render_area: ShipRenderArea,
    
    // Selected module
    selection: Option<(ModuleRef, module::TargetMode)>,
    
    // Current state of targeting
    beam_targeting_state: Option<Vec2f>,
    
    mouse_x: f64,
    mouse_y: f64,
    
    // Textures
    small_ship_icon: Texture,
    medium_ship_icon: Texture,
    big_ship_icon: Texture,
    
    stats_labels: StatsLabels,
    
    module_icons: ModuleIcons,
    
    // Space background
    space_bg: SpaceStars,
    
    // Star map stuff
    star_map_button: TextButton,
    star_map_gui: StarMapGui,
    show_star_map: bool,
    
    // Logout button
    logout_button: TextButton,

    // targets
    target_icons: Vec<TargetIcon>,
}

impl SpaceGui {
    pub fn new(asset_store: &AssetStore, context: &BattleContext, sectors: Vec<SectorData>, my_ship: ShipIndex) -> SpaceGui {
        // Set up the render area
        //let target = RenderTexture::new(500, 500, false).expect("Failed to create render texture");
        //let texture = target.get_texture().expect("Failed to get render texture's texture");
        let x = 1280.0 - 5.0 - 560.0;
        let y = 128.0;
        let ship = context.ships_iter().filter(|ship| ship.borrow().index != my_ship).next().map(|ship| ship.borrow().index);
        let render_area = ShipRenderArea {
            ship: ship,
            x: x,
            y: y,
            width: 560.0,
            height: (720.0 - 20.0) - y,
            //target: target,
            //texture: texture,
        };

        let target_icons =
            context.ships_iter()
            .filter(|ship| ship.borrow().index != my_ship)
            .take(5)
            .map(|ship| TargetIcon { ship: ship.borrow().index })
            .collect();
    
        SpaceGui {
            plans: ShipPlans::new(),
            render_area: render_area,
            selection: None,
            beam_targeting_state: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            
            small_ship_icon: Texture::from_path(&Path::new("content/textures/gui/small_target.png")).unwrap(),
            medium_ship_icon: Texture::from_path(&Path::new("content/textures/gui/medium_target.png")).unwrap(),
            big_ship_icon: Texture::from_path(&Path::new("content/textures/gui/big_target.png")).unwrap(),
            
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
            star_map_gui: StarMapGui::new(sectors),
            show_star_map: false,
            
            logout_button: TextButton::new("logout".to_string(), 20, [550.0, 100.0], [120.0, 40.0]),

            target_icons: target_icons,
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, bc: &BattleContext, e: &E, client_ship: &ShipRef) {
        use event::*;
        
        if client_ship.borrow().state.get_hp() == 0 {
            return;
        }
    
        e.mouse_cursor(|x, y| {
            self.mouse_x = x;
            self.mouse_y = y;
        });
        
        self.star_map_button.event(e, [self.mouse_x, self.mouse_y]);
        if self.star_map_button.get_clicked() {
            self.show_star_map = true;
        }
        
        if self.show_star_map {
            if let Some(star_map_result) = self.star_map_gui.event(e, [self.mouse_x - 200.0, self.mouse_y - 200.0]) {
                match star_map_result {
                    StarMapAction::Jump(sector) => {
                        client_ship.borrow_mut().target_sector = Some(sector);
                        self.show_star_map = false;
                    },
                    StarMapAction::Close => {
                        self.show_star_map = false;
                    },
                }
            }
        } else {
            e.press(|button| {
                match button {
                    Button::Keyboard(key) => self.on_key_pressed(key), 
                    Button::Mouse(button) => {
                        let (mouse_x, mouse_y) = (self.mouse_x, self.mouse_y);
                        match button {
                            mouse::MouseButton::Left => self.on_mouse_left_pressed(bc, mouse_x, mouse_y, client_ship),
                            mouse::MouseButton::Right => self.on_mouse_right_pressed(bc, mouse_x, mouse_y, client_ship),
                            _ => {},
                        }
                    },
                }
            });
        }
        
        self.logout_button.event(e, [self.mouse_x, self.mouse_y]);
        if self.logout_button.get_clicked() {
            // TODO: Logout
        }
    }
    
    pub fn draw_simulating(
        &mut self,
        bc: &BattleContext,
        context: &Context,
        gl: &mut Gl,
        glyph_cache: &mut GlyphCache,
        asset_store: &AssetStore,
        sim_effects: &mut SimEffects,
        client_ship: &mut Ship,
        time: f64,
        dt: f64
    )
    {
        use graphics::*;
        
        // Clear the screen
        clear([0.0; 4], gl);
        
        self.draw_screen(bc, context, gl, glyph_cache, asset_store, sim_effects, client_ship, time, dt);
        
        // Draw plan timer bar
        let plan_timer =
            if time < 2.5 {
                (2.5 + time) / 5.0
            } else {
                (time - 2.5) / 5.0
            };
        
        Rectangle::new([0.0, 0.0, 1.0, 0.5])
                .draw(
                    [550.0, 10.0, 100.0, 32.0],
                    &context.draw_state, context.transform,
                    gl
                );
        Rectangle::new([0.0, 0.0, 1.0, 1.0])
                .draw(
                    [550.0, 10.0, plan_timer * 100.0, 32.0],
                    &context.draw_state, context.transform,
                    gl
                );
        
        if self.show_star_map {
            self.star_map_gui.draw(&context.trans(200.0, 200.0), gl, glyph_cache);
        }
    }
    
    fn draw_screen(
        &mut self,
        bc: &BattleContext,
        context: &Context,
        gl: &mut Gl,
        glyph_cache: &mut GlyphCache,
        asset_store: &AssetStore,
        sim_effects: &mut SimEffects,
        client_ship: &mut Ship,
        time: f64,
        dt: f64,
    )
    {
        use graphics::*;
        
        // Draw the space background
        self.space_bg.update(dt);
        self.space_bg.draw(context, gl);
        
        // Draw player ship
        draw_ship(&context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl, asset_store, sim_effects, client_ship, time);
        client_ship.draw_module_powered_icons(&context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y), gl, &self.module_icons);
        draw_stats(context, gl, glyph_cache, &self.stats_labels, client_ship.deref(), true);
    
        let mut enemy_alive = false;
        if let Some(ship) = self.render_area.ship {
            // TODO clear render texture
            
            Rectangle::new([1.0, 0.7, 0.2, 0.5])
                .draw(
                    [self.render_area.x, self.render_area.y, self.render_area.width, self.render_area.height],
                    &context.draw_state, context.transform,
                    gl
                );
        
            {
                let context = context.trans(self.render_area.x, self.render_area.y);
                
                draw_ship(&context.trans(ENEMY_OFFSET_X, ENEMY_OFFSET_Y), gl, asset_store, sim_effects, ship.get(bc).borrow().deref(), time);
                draw_stats(&context.trans(0.0, 400.0), gl, glyph_cache, &self.stats_labels, ship.get(bc).borrow().deref(), false);
            }
            
            // TODO draw render texture
        
            if ship.get(bc).borrow().state.get_hp() > 0 {
                enemy_alive = true;
            }
        }
        
        if let Some(ref selection) = self.selection {
            let &(ref selected_module, ref target_mode) = selection;
            
            // Highlight selected module
            let x = self.mouse_x - SHIP_OFFSET_X;
            let y = self.mouse_y - SHIP_OFFSET_Y;
            
            let module_borrowed = selected_module.borrow();

            let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
            let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
            let (module_x, module_y, module_w, module_h) = (module_x as f64, module_y as f64, module_w as f64, module_h as f64);
        
            {
                let context = context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y);
                Rectangle::new([0.0, 1.0, 0.0, 0.5])
                    .draw(
                        [module_x, module_y, module_w, module_h],
                        &context.draw_state, context.transform,
                        gl
                    );
            }
            
            // Draw beam targeting visual
            match target_mode {
                &module::TargetMode::Beam(beam_length) => {
                    if let Some(beam_start) = self.beam_targeting_state {
                        let x = self.mouse_x - self.render_area.x - ENEMY_OFFSET_X;
                        let y = self.mouse_y - self.render_area.y - ENEMY_OFFSET_Y;
                        let beam_length = (beam_length as f64) * 48.0;
                        
                        let beam_end = calculate_beam_end(beam_start, Vec2 { x: x, y: y }, beam_length);
                        
                        let context = context.trans(self.render_area.x + ENEMY_OFFSET_X, self.render_area.y + ENEMY_OFFSET_Y);
                        
                        // Draw targeting circles
                        if let Some(ship) = self.render_area.ship {
                            ship.get(bc).borrow().beam_hits(beam_start, beam_end, |_, circle_pos, radius, hit| {
                                let circle =
                                    if let Some(hit_dist) = hit {
                                        Ellipse::new([1.0, 0.0, 0.0, 0.5])
                                    } else {
                                        Ellipse::new([0.0, 0.0, 1.0, 0.5])
                                    };
                                
                                let size = radius * 2.0;
                                
                                circle.draw(
                                    [circle_pos.x - radius, circle_pos.y - radius, size, size],
                                    &context.draw_state, context.transform,
                                    gl
                                );
                            });
                        }
                        
                        Line::new([1.0, 0.0, 0.0, 1.0], 2.0)
                            .draw(
                                [beam_start.x, beam_start.y, beam_end.x, beam_end.y],
                                &context.draw_state, context.transform,
                                gl
                            );
                    }
                },
                &module::TargetMode::TargetModule => {
                    if let Some(ship) = self.render_area.ship {
                        // Highlight target modules the user mouses-over red
                        let x = self.mouse_x - self.render_area.x - ENEMY_OFFSET_X;
                        let y = self.mouse_y - self.render_area.y - ENEMY_OFFSET_Y;

                        apply_to_module_if_point_inside(ship.get(bc).borrow_mut().deref_mut(), x, y, |_, _, _, module_borrowed| {
                            let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
                            let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
                            let (module_x, module_y, module_w, module_h) = (module_x as f64, module_y as f64, module_w as f64, module_h as f64);
                            
                            let context = context.trans(self.render_area.x + ENEMY_OFFSET_X, self.render_area.y + ENEMY_OFFSET_Y);

                            Rectangle::new([1.0, 0.0, 0.0, 0.5])
                                .draw(
                                    [module_x, module_y, module_w, module_h],
                                    &context.draw_state, context.transform,
                                    gl
                                );
                        });
                    }
                },
                _ => { },
            }
        } else {
            // If not currently selecting a module, highlight modules the user mouses-over
            let x = self.mouse_x - SHIP_OFFSET_X;
            let y = self.mouse_y - SHIP_OFFSET_Y;

            apply_to_module_if_point_inside(client_ship, x, y, |_, ship_state, _, module_borrowed| {
                let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
                let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
                let (module_x, module_y, module_w, module_h) = (module_x as f64, module_y as f64, module_w as f64, module_h as f64);
            
                let context = context.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y);
                if module_borrowed.get_base().plan_powered {
                    Rectangle::new([0.0, 0.0, 1.0, 0.5])
                        .draw(
                            [module_x, module_y, module_w, module_h],
                            &context.draw_state, context.transform,
                            gl
                        );
                } else if ship_state.can_plan_activate_module(module_borrowed.get_base()) {
                    Rectangle::new([1.0, 1.0, 0.0, 0.5])
                        .draw(
                            [module_x, module_y, module_w, module_h],
                            &context.draw_state, context.transform,
                            gl
                        );
                }
            });
        }
        
        self.star_map_button.draw(context, gl, glyph_cache);
        self.logout_button.draw(context, gl, glyph_cache);

        // Draw target icons
        for (i, icon) in self.target_icons.iter().enumerate() {
            let i = i as f64;
            
            let icon_x = 715.0+(i*100.0);
            let icon_y = 5.0;
            
            let ref context = context.trans(icon_x, icon_y);
            
            icon.draw(bc, context, gl, glyph_cache, asset_store);
            
            match self.render_area.ship {
                Some(ship) if ship == icon.ship => {
                    Rectangle::new([1.0, 0.0, 0.0, 0.5])
                        .draw([0.0, 0.0, 96.0, 96.0], &context.draw_state, context.transform, gl);
                },
                _ => {
                    if self.mouse_x >= icon_x && self.mouse_x <= icon_x+96.0 &&
                        self.mouse_y >= icon_y && self.mouse_y <= icon_y+96.0
                    {
                        Rectangle::new([0.0, 0.0, 1.0, 0.5])
                            .draw([0.0, 0.0, 96.0, 96.0], &context.draw_state, context.transform, gl);
                    }
                },
            }
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    fn on_mouse_left_pressed(&mut self, bc: &BattleContext, x: f64, y: f64, client_ship: &ShipRef) {
        // Handle module plan powering and selection
        if self.selection.is_none() {
            let x = x - SHIP_OFFSET_X;
            let y = y - SHIP_OFFSET_Y;
            
            apply_to_module_if_point_inside(client_ship.borrow_mut().deref_mut(), x, y, |_, ship_state, module, module_borrowed| {
                if module_borrowed.get_base().plan_powered {
                    if let Some(target_mode) = module_borrowed.get_target_mode() {
                        // Select this module to begin targeting
                        self.selection = Some((module.clone(), target_mode));
                    }
                } else if ship_state.can_plan_activate_module(module_borrowed.get_base()) {
                    ship_state.plan_activate_module(module_borrowed.get_base_mut());
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
                    
                    if let Some(ship) = self.render_area.ship {
                        if !ship.get(bc).borrow().jumping && !ship.get(bc).borrow().exploding {
                            apply_to_module_if_point_inside(ship.get(bc).borrow_mut().deref_mut(), x, y, |ship_index, _, _, module| {
                                selected_module.borrow_mut().get_base_mut().plan_target =
                                    Some(module::Target {
                                        ship: ship_index,
                                        data: module::TargetData::TargetModule(module.get_base().index),
                                    });
                                clear_selection = true;
                            });
                        }
                    }
                },
                module::TargetMode::OwnModule => {
                    let x = x - SHIP_OFFSET_X;
                    let y = y - SHIP_OFFSET_Y;
                    
                    apply_to_module_if_point_inside(client_ship.borrow_mut().deref_mut(), x, y, |ship_index, _, _, module| {
                        selected_module.borrow_mut().get_base_mut().plan_target =
                            Some(module::Target {
                                ship: ship_index,
                                data: module::TargetData::OwnModule(module.get_base().index),
                            });
                        clear_selection = true;
                    });
                },
                module::TargetMode::Beam(beam_length) => {
                    let x = x - self.render_area.x - ENEMY_OFFSET_X;
                    let y = y - self.render_area.y - ENEMY_OFFSET_Y;
                    let beam_length = (beam_length as f64) * 48.0;
                    
                    if x >= 0.0 && y >= 0.0 {
                        if let Some(ship) = self.render_area.ship {
                            if !ship.get(bc).borrow().jumping && !ship.get(bc).borrow().exploding {
                                if let Some(beam_start) = self.beam_targeting_state {
                                    let beam_end = calculate_beam_end(beam_start, Vec2 { x: x, y: y }, beam_length);
                                    selected_module.borrow_mut().get_base_mut().plan_target =
                                        Some(module::Target {
                                            ship: ship.get(bc).borrow().index,
                                            data: module::TargetData::Beam(beam_start, beam_end),
                                        });
                                    clear_selection = true;
                                    self.beam_targeting_state = None;
                                } else {
                                    self.beam_targeting_state = Some(Vec2 { x: x, y: y });
                                }
                            }
                        }
                    }
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

                if let Some(ship) = self.render_area.ship { // switching to a new ship
                    if ship.get(bc).borrow().index != icon.ship {
                        should_change = true;
                    } else {
                        // do nothing
                    }
                } 
                if should_change {
                    self.render_area.ship = Some(icon.ship);
                    break;
                }
            }
        }
    }
    
    fn on_mouse_right_pressed(&mut self, bc: &BattleContext, x: f64, y: f64, client_ship: &ShipRef) {
        let mut module_was_deactivated = false;
    
        if self.selection.is_none() {
            let x = x - SHIP_OFFSET_X;
            let y = y - SHIP_OFFSET_Y;
            
            apply_to_module_if_point_inside(client_ship.borrow_mut().deref_mut(), x, y, |_, ship_state, _, module_borrowed| {
                if module_borrowed.get_base().plan_powered {
                    ship_state.plan_deactivate_module(module_borrowed.get_base_mut());
                }
                module_was_deactivated = true;
            });
        }
        
        if !module_was_deactivated {
            self.selection = None;
            self.beam_targeting_state = None;
        }
    }
    
    pub fn try_lock(&mut self, ship: ShipIndex) {
        if self.render_area.ship.is_none() {
            self.render_area.ship = Some(ship);
        }
        
        if self.target_icons.len() < 5 {
            self.target_icons.push(TargetIcon { ship: ship });
        }
    }
    
    pub fn remove_lock(&mut self, ship: ShipIndex) {
        if self.render_area.ship.is_some() && self.render_area.ship.unwrap() == ship {
            self.render_area.ship = None;
        }
        
        self.target_icons.retain(|i| i.ship != ship);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Applies function to a module in the ship if the mouse is over the module.
/// Returns whether or not the function was applied.
pub fn apply_to_module_if_point_inside<F>(ship: &mut Ship, x: f64, y: f64, mut f: F)
    where
        F: FnMut(ShipIndex, &mut ShipState, &ModuleRef, &mut ModuleBox)
{
    for module in ship.modules.iter() {
        let mut module_borrowed = module.borrow_mut();
    
        // Get module position and size on screen
        let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
        let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
        let (module_x, module_y, module_w, module_h) = (module_x as f64, module_y as f64, module_w as f64, module_h as f64);
        if x >= module_x && x <= module_x+module_w && y >= module_y && y <= module_y+module_h {
            f(ship.index, &mut ship.state, module, module_borrowed.deref_mut());
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn calculate_beam_end(beam_start: Vec2f, mouse_pos: Vec2f, beam_length: f64) -> Vec2f {
    let beam_vec = (mouse_pos - beam_start).normalize() * beam_length;
    beam_start + beam_vec
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct TargetIcon {
    ship: ShipIndex,
}

impl TargetIcon {
    fn draw(&self, bc: &BattleContext, context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache, asset_store: &AssetStore) {
        use graphics::*;
        use graphics::text::Text;
    
        let ship = self.ship.get(bc).borrow();
        
        let icon =
            match ship.get_height() {
                1...2 => asset_store.get_texture_str("gui/small_target.png"),
                3 => asset_store.get_texture_str("gui/medium_target.png"),
                4...255 => asset_store.get_texture_str("gui/big_target.png"),
                _ => unreachable!(),
            };
        
        let (icon_w, icon_h) = icon.get_size();
        let (half_icon_w, half_icon_h) = ((icon_w/2) as f64, (icon_h/2) as f64);
        
        image(icon.deref(), context.trans(48.0 - half_icon_w, 34.0 - half_icon_h).transform, gl);
        
        // Draw stats bars
        
        let hp = ship.state.hp as f64;
        //let max_hp = ship.state.max_hp as f64;
        let shields = ship.state.shields as f64;
        let max_shields = ship.state.max_shields as f64;
        let power = ship.state.available_power() as f64;
        let max_power = ship.state.max_power as f64;
        
        // HP
        //Rectangle::new([0.0, 1.0, 0.0, 0.5])
        //    .draw([2.0, 72.0, max_hp, 3.0], &context.draw_state, context.transform, gl);
        Rectangle::new([0.0, 1.0, 0.0, 1.0])
            .draw([2.0, 72.0, hp, 3.0], &context.draw_state, context.transform, gl);
        
        // Shields
        Rectangle::new([0.0, 0.0, 1.0, 0.5])
            .draw([2.0, 76.0, max_shields, 3.0], &context.draw_state, context.transform, gl);
        Rectangle::new([0.0, 0.0, 1.0, 1.0])
            .draw([2.0, 76.0, shields, 3.0], &context.draw_state, context.transform, gl);
        
        // Power
        Rectangle::new([1.0, 1.0, 0.0, 0.5])
            .draw([2.0, 80.0, max_power, 3.0], &context.draw_state, context.transform, gl);
        Rectangle::new([1.0, 1.0, 0.0, 1.0])
            .draw([2.0, 80.0, power, 3.0], &context.draw_state, context.transform, gl);
        
        
        // Draw ship's name
        {
            let context = context.trans(2.0, 94.0);
            Text::colored([1.0; 4], 10).draw(
                ship.name.as_slice(),
                glyph_cache,
                &context.draw_state, context.transform,
                gl,
            );
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ShipRenderArea {
    ship: Option<ShipIndex>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    //target: RenderTexture,
    //texture: Texture,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn draw_ship(context: &Context, gl: &mut Gl, asset_store: &AssetStore, sim_effects: &mut SimEffects, ship: &Ship, time: f64) {
    ship.draw(context, gl, asset_store);
    sim_effects.update(context, gl, ship.id, time);
    
    if !ship.exploding {
        ship.draw_module_hp(context, gl);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn draw_stats(context: &Context, gl: &mut Gl, glyph_cache: &mut GlyphCache, stats_labels: &StatsLabels, ship: &Ship, is_client_ship: bool) {
    use std::cmp;

    use graphics::*;
    use graphics::text::Text;
    
    let hp_rect = Rectangle::new([0.0, 1.0, 0.0, 1.0]);
    let shield_rect = Rectangle::new([0.0, 0.0, 1.0, 1.0]);
    let power_rect = Rectangle::new([1.0, 1.0, 0.0, 1.0]);
    
    {
        let context = context.trans(5.0, 5.0 + 14.0);
        for i in 0..ship.state.get_hp() {
            hp_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.draw_state, context.transform, gl);
        }
    }
    
    {
        let context = context.trans(5.0, 5.0 + 52.0);
        for i in 0..ship.state.shields {
            shield_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.draw_state, context.transform, gl);
        }
    }
    
    {
        let context = context.trans(5.0, 5.0 + 90.0);
        if is_client_ship {
            let used_power_rect = Rectangle::new([1.0, 1.0, 0.0, 0.5]);
            let new_power_rect = Rectangle::new([0.0, 1.0, 0.0, 1.0]);
            
            for i in 0..cmp::min(ship.state.available_plan_power(), ship.state.available_power()) {
                power_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.draw_state, context.transform, gl);
            }
        
            if ship.state.available_plan_power() < ship.state.available_power() {
                for i in ship.state.available_plan_power()..ship.state.available_power() {
                    used_power_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.draw_state, context.transform, gl);
                }
            } else if ship.state.available_plan_power() > ship.state.available_power() {
                for i in ship.state.available_power()..ship.state.available_plan_power() {
                    new_power_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.draw_state, context.transform, gl);
                }
            }
        } else {
            for i in 0..ship.state.available_power() {
                power_rect.draw([(i as f64)*10.0, 0.0, 8.0, 16.0], &context.draw_state, context.transform, gl);
            }
        }
    }
    
    // Draw labels for hp, shields and power meters
    image(&stats_labels.hp_texture, context.trans(5.0, 4.0).transform, gl);
    image(&stats_labels.shield_texture, context.trans(5.0, 42.0).transform, gl);
    image(&stats_labels.power_texture, context.trans(5.0, 80.0).transform, gl);
    
    {
        let context = context.trans(5.0, 160.0);
        Text::colored([1.0; 4], 30).draw(
            ship.name.as_slice(),
            glyph_cache,
            &context.draw_state, context.transform,
            gl,
        );
    }
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
        for _ in 0..stars.capacity() {
            let mut layer = Vec::with_capacity(50);
            for _ in 0u8..20 {
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
                star_rect.draw(
                    [star.position[0], star.position[1], star.size, star.size],
                    &context.draw_state, context.transform,
                    gl
                );
            }
        }
    }
}
