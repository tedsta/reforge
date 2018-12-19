use rand::Rng;
use rand;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use ggez::{
    Context, GameResult,
    event::{Event, Keycode, MouseButton},
    graphics::{self, DrawMode, DrawParam, Image, Point2, Rect},
};

use asset_store::AssetStore;
use battle_context::BattleContext;
//use chat::{ChatGui, ChatGuiAction};
use client_context::ReforgeClientContext;
use gui::{TextButton, SpriteButton};
use module;
use module::{IModule, Module, ModuleIndex};
//use nav_map_gui::{NavMapGui, NavMapGuiAction};
use net::ClientId;
use sector_data::SectorData;
use ship::{Ship, ShipId, ShipIndex, ShipPlans, ShipState};
use sim::SimEffects;
use star_map::{StarMapGui, StarMapGuiAction};
use util::with_translate;
use vec::{Vec2, Vec2f};

static SHIP_OFFSET_X: f32 = 80.0;
static SHIP_OFFSET_Y: f32 = 170.0;

static ENEMY_OFFSET_X: f32 = 80.0;
static ENEMY_OFFSET_Y: f32 = 50.0;

pub enum SpaceGuiAction {
    Chat(String),
    Logout,
}

pub struct ModuleIcons {
    pub power_on_texture: Image,
    pub power_off_texture: Image,
}

pub struct StatsLabels {
    hp_texture: Image,
    shield_texture: Image,
    power_texture: Image,
}

pub struct SpaceGui {
    // Plans for player's ship
    pub plans: ShipPlans,

    // The target ships' render areas
    render_area: ShipRenderArea,
    
    // Selected module
    selection: Option<(ModuleIndex, module::TargetMode)>,
    
    // Current state of targeting
    beam_targeting_state: Option<Vec2f>,
    
    mouse_pos: Vec2f,
    
    // Textures
    overlay_hud: Image,
    
    small_ship_icon: Image,
    medium_ship_icon: Image,
    big_ship_icon: Image,
    
    stats_labels: StatsLabels,
    
    module_icons: ModuleIcons,
    
    // Space background
    //space_bg: SpaceStars,
    
    // Star map stuff
    star_map_button: SpriteButton,
    star_map_gui: StarMapGui,
    show_star_map: bool,
    
    // Nav map stuff
    nav_map_button: SpriteButton,
    //nav_map_gui: NavMapGui,
    show_nav_map: bool,
    
    // Chat
    chat_button: SpriteButton,
    chat_gui_pos: Vec2f,
    //pub chat_gui: &'a mut ChatGui,
    
    // Logout button
    logout_button: SpriteButton,

    // targets
    target_icons: Vec<TargetIcon>,
}

impl SpaceGui {
    pub fn new(gtx: &mut ReforgeClientContext,
               bc: &BattleContext,
               ctx: &mut Context,
               //chat_gui: &'a mut ChatGui,
               my_ship: ShipIndex) -> GameResult<SpaceGui> {
        // Set up the render area
        //let target = RenderTexture::new(500, 500, false).expect("Failed to create render texture");
        //let texture = target.get_texture().expect("Failed to get render texture's texture");
        let x = 1280.0 - 5.0 - 560.0;
        let y = 128.0;
        let ship = bc.ships_iter().filter(|ship| ship.index != my_ship).next().map(|ship| ship.index);
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
            bc.ships_iter()
            .filter(|ship| ship.index != my_ship)
            .take(5)
            .map(|ship| TargetIcon { ship: ship.index })
            .collect();
    
        Ok(SpaceGui {
            plans: my_ship.get(bc).create_plans(),
            render_area: render_area,
            selection: None,
            beam_targeting_state: None,
            mouse_pos: Vec2::new(0.0, 0.0),
            
            overlay_hud: Image::new(ctx, "/textures/gui/overlay_hud.png")?,
            
            small_ship_icon: Image::new(ctx, "/textures/gui/small_target.png")?,
            medium_ship_icon: Image::new(ctx, "/textures/gui/medium_target.png")?,
            big_ship_icon: Image::new(ctx, "/textures/gui/big_target.png")?,
            
            stats_labels: StatsLabels {
                hp_texture: Image::new(ctx, "/textures/gui/hp_text.png")?,
                shield_texture: Image::new(ctx, "/textures/gui/shield_text.png")?,
                power_texture: Image::new(ctx, "/textures/gui/power_text.png")?,
            },
            
            module_icons: ModuleIcons {
                power_on_texture: Image::new(ctx, "/textures/gui/power_on_icon.png")?,
                power_off_texture: Image::new(ctx, "/textures/gui/power_off_icon.png")?,
            },
            
            //space_bg: SpaceStars::new(),
            
            star_map_button: SpriteButton::new(ctx, "/textures/gui/starmap.png", 3, 1, [488.0, 1.0])?,
            star_map_gui: StarMapGui::new(gtx)?,
            show_star_map: false,
            
            nav_map_button: SpriteButton::new(ctx, "/textures/gui/target.png", 3, 1, [626.0, 7.0])?,
            //nav_map_gui: NavMapGui::new(&gtx.asset_store),
            show_nav_map: false,
            
            chat_button: SpriteButton::new(ctx, "/textures/gui/chat.png", 3, 1, [86.0, 654.0])?,
            chat_gui_pos: Vec2::new(437.0, 608.0),
            //chat_gui: chat_gui,
            
            logout_button: SpriteButton::new(ctx, "/textures/gui/logout.png", 3, 1, [16.0, 14.0])?,
            
            target_icons: target_icons,
        })
    }
    
    pub fn event(
        &mut self,
        gtx: &mut ReforgeClientContext, bc: &mut BattleContext,
        e: &Event, client_ship: ShipIndex, time: f64)
        -> Option<SpaceGuiAction>
    {
        use Event::*;

        if client_ship.get(bc).state.get_hp() == 0 {
            return None;
        }

        match *e {
            MouseMotion { x, y, .. } => {
                self.mouse_pos = Vec2::new(x as f64, y as f64);
            },
            _ => { },
        }
        
        /*if let Some(chat_action) = self.chat_gui.event(e, self.mouse_pos - self.chat_gui_pos) {
            match chat_action {
                ChatGuiAction::SendMsg(msg) => {
                    return Some(SpaceGuiAction::Chat(msg));
                },
            }
        }*/
        
        self.chat_button.event(e, self.mouse_pos);
        if self.chat_button.get_clicked() {
            // do something
        }
        
        self.star_map_button.event(e, self.mouse_pos);
        if self.star_map_button.get_clicked() {
            self.show_star_map = !self.show_star_map;
        }
        
        /*self.nav_map_button.event(e, [self.mouse_pos.x, self.mouse_pos.y]);
        if self.nav_map_button.get_clicked() {
            self.show_nav_map = !self.show_nav_map;
        }*/
        
        if self.show_star_map {
            if let Some(star_map_result) = self.star_map_gui.event(
                gtx, e, self.mouse_pos - Vec2::new(200.0, 200.0)
            ) {
                match star_map_result {
                    StarMapGuiAction::Jump(sector) => {
                        self.plans.target_sector = Some(sector);
                        self.show_star_map = false;
                    },
                    StarMapGuiAction::Close => {
                        self.show_star_map = false;
                    },
                }
            }
        } else if self.show_nav_map {
            /*if let Some(nav_map_result) =
                self.nav_map_gui.event(e, [self.mouse_pos.x - 200.0, self.mouse_pos.y - 200.0],
                                       bc, client_ship, time)
            {
                match nav_map_result {
                    NavMapGuiAction::Close => {
                        self.show_nav_map = false;
                    },
                }
            }*/
        } else {
            match *e {
                KeyDown { keycode: Some(keycode), .. } => {
                    self.on_key_pressed(keycode);
                },
                MouseButtonDown { mouse_btn, .. } => {
                    match mouse_btn {
                        MouseButton::Left => {
                            self.on_mouse_left_pressed(bc, self.mouse_pos, client_ship.get(bc));
                        },
                        MouseButton::Left => {
                            self.on_mouse_right_pressed(bc, self.mouse_pos, client_ship.get(bc));
                        },
                        _ => { },
                    }
                }
                _ => { }
            }
        }
        
        self.logout_button.event(e, self.mouse_pos);
        if self.logout_button.get_clicked() {
            return Some(SpaceGuiAction::Logout);
        }
        
        None
    }
    
    pub fn draw_simulating(
        &mut self,
        gtx: &mut ReforgeClientContext,
        bc: &BattleContext,
        ctx: &mut Context,
        sim_effects: &mut SimEffects,
        client_ship: &Ship,
        time: f64,
        dt: f64)
        -> GameResult<()>
    {
        self.draw_screen(gtx, bc, ctx, sim_effects, client_ship, time, dt)?;
        
        // Draw plan timer bar
        let plan_timer =
            if time < 2.5 {
                (2.5 + time) / 5.0
            } else {
                (time - 2.5) / 5.0
            } as f32;
        graphics::set_color(ctx, [0.0, 0.0, 1.0, 0.5].into())?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(550.0, 10.0, 100.0, 32.0))?;
        graphics::set_color(ctx, [0.0, 0.0, 1.0, 1.0].into())?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(550.0, 10.0, plan_timer * 100.0, 32.0))?;
        
        //self.chat_gui.draw(&ctx.trans(self.chat_gui_pos.x, self.chat_gui_pos.y), gl, glyph_cache);
        
        if self.show_star_map {
            with_translate(ctx, Point2::new(200.0, 200.0), |ctx| {
                self.star_map_gui.draw(gtx, ctx)
            })?;
        }
        
        if self.show_nav_map {
            //self.nav_map_gui.draw(&ctx.trans(200.0, 200.0), gl, glyph_cache, bc, client_ship, time);
        }

        Ok(())
    }
    
    fn draw_screen(
        &mut self,
        gtx: &mut ReforgeClientContext,
        bc: &BattleContext,
        ctx: &mut Context,
        sim_effects: &mut SimEffects,
        client_ship: &Ship,
        time: f64,
        dt: f64)
        -> GameResult<()>
    {
        use graphics::*;
        // Draw the space background
        // TODO
        //self.space_bg.update(dt);
        //self.space_bg.draw(ctx, gl);
        
        graphics::draw_ex(ctx, &self.overlay_hud, Default::default())?;
        
        // Draw player ship
        with_translate(ctx, Point2::new(SHIP_OFFSET_X, SHIP_OFFSET_Y), |ctx| -> GameResult<()> {
            draw_ship(gtx, ctx, sim_effects, client_ship, time);
            client_ship.draw_module_powered_icons(ctx, &self.module_icons, &self.plans);
            Ok(())
        })?;
        draw_stats(ctx, &self.stats_labels, &self.plans, client_ship, true);
    
        let mut enemy_alive = false;
        if let Some(ship) = self.render_area.ship {
            // TODO clear render texture
            
            graphics::set_color(ctx, [1.0, 0.7, 0.2, 0.5].into())?;
            graphics::rectangle(
                ctx, DrawMode::Fill,
                Rect::new(self.render_area.x, self.render_area.y,
                          self.render_area.width, self.render_area.height))?;
            graphics::set_color(ctx, [1.0; 4].into())?;
        
            with_translate(
                ctx, Point2::new(self.render_area.x, self.render_area.y),
                |ctx| -> GameResult<()> {
                    with_translate(
                        ctx, Point2::new(ENEMY_OFFSET_X, ENEMY_OFFSET_Y),
                        |ctx| draw_ship(gtx, ctx, sim_effects, ship.get(bc), time))?;
                    with_translate(
                        ctx, Point2::new(0.0, 400.0),
                        |ctx| draw_stats(
                            ctx, &self.stats_labels, &self.plans, ship.get(bc), false))?;
                    Ok(())
                })?;
            
            // TODO draw render texture
        
            if ship.get(bc).state.get_hp() > 0 {
                enemy_alive = true;
            }
        }
        
        /*if let Some(ref selection) = self.selection {
            let &(selected_module, ref target_mode) = selection;
            
            let selected_module = selected_module.get(client_ship);
            
            // Highlight selected module
            {
                let Vec2{x: module_x, y: module_y} = selected_module.get_render_position();
                
                let ctx = ctx.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y);
                
                for x in (0..selected_module.shape.side()) {
                    for y in (0..selected_module.shape.side()) {
                        if selected_module.shape.get(x, y) == b'#' {
                            let offset_x = x as f64 * 48.0;
                            let offset_y = y as f64 * 48.0;
                            if self.plans.module_plans(selected_module.index).active {
                                Rectangle::new([0.0, 1.0, 0.0, 0.5])
                                    .draw(
                                        [module_x + offset_x, module_y + offset_y, 48.0, 48.0],
                                        &ctx.draw_state, ctx.transform,
                                        gl
                                    );
                            } else if self.plans.can_plan_activate_module(&client_ship.state, selected_module) {
                                Rectangle::new([1.0, 1.0, 0.0, 0.5])
                                    .draw(
                                        [module_x + offset_x, module_y + offset_y, 48.0, 48.0],
                                        &ctx.draw_state, ctx.transform,
                                        gl
                                    );
                            }
                        }
                    }
                }
            }
            
            let x = self.mouse_pos.x - SHIP_OFFSET_X;
            let y = self.mouse_pos.y - SHIP_OFFSET_Y;
            
            // Draw beam targeting visual
            match target_mode {
                &module::TargetMode::Beam(beam_length) => {
                    let ctx = ctx.trans(self.render_area.x + ENEMY_OFFSET_X, self.render_area.y + ENEMY_OFFSET_Y);
                    
                    if let Some(ship) = self.render_area.ship {
                        let beam = self.beam_targeting_state.map(|beam_start| {
                                let x = self.mouse_pos.x - self.render_area.x - ENEMY_OFFSET_X;
                                let y = self.mouse_pos.y - self.render_area.y - ENEMY_OFFSET_Y;
                                let beam_length = (beam_length as f64) * 48.0;
                                
                                let beam_end = calculate_beam_end(beam_start, Vec2 { x: x, y: y }, beam_length);
                                
                                (beam_start, beam_end)
                            });
                        
                        // Draw targeting circles
                        ship.get(bc).beam_hits(beam, |_, circle_pos, radius, hit| {
                            let circle =
                                if let Some(hit_dist) = hit {
                                    Ellipse::new([1.0, 0.0, 0.0, 0.5])
                                } else {
                                    Ellipse::new([0.0, 0.0, 1.0, 0.5])
                                };
                            
                            let size = radius * 2.0;
                            
                            circle.draw(
                                [circle_pos.x - radius, circle_pos.y - radius, size, size],
                                &ctx.draw_state, ctx.transform,
                                gl
                            );
                        });
                        
                        if let Some((beam_start, beam_end)) = beam {
                            Line::new([1.0, 0.0, 0.0, 1.0], 2.0)
                                .draw(
                                    [beam_start.x, beam_start.y, beam_end.x, beam_end.y],
                                    &ctx.draw_state, ctx.transform,
                                    gl
                                );
                        }
                    }
                },
                &module::TargetMode::TargetModule => {
                    if let Some(ship) = self.render_area.ship {
                        // Highlight target modules the user mouses-over red
                        let x = self.mouse_pos.x - self.render_area.x - ENEMY_OFFSET_X;
                        let y = self.mouse_pos.y - self.render_area.y - ENEMY_OFFSET_Y;

                        apply_to_module_if_point_inside(ship.get(bc), x, y, |_, _, module| {
                            let Vec2{x: module_x, y: module_y} = module.get_render_position();
                            
                            let ctx = ctx.trans(self.render_area.x + ENEMY_OFFSET_X, self.render_area.y + ENEMY_OFFSET_Y);
                            
                            for x in (0..module.shape.side()) {
                                for y in (0..module.shape.side()) {
                                    if module.shape.get(x, y) == b'#' {
                                        let offset_x = x as f64 * 48.0;
                                        let offset_y = y as f64 * 48.0;
                                        
                                        Rectangle::new([1.0, 0.0, 0.0, 0.5])
                                            .draw(
                                                [module_x + offset_x, module_y + offset_y, 48.0, 48.0],
                                                &ctx.draw_state, ctx.transform,
                                                gl
                                            );
                                    }
                                }
                            }
                        });
                    }
                },
                &module::TargetMode::OwnModule => {
                    // Highlight target modules the user mouses-over red
                    let x = self.mouse_pos.x - SHIP_OFFSET_X;
                    let y = self.mouse_pos.y - SHIP_OFFSET_Y;

                    apply_to_module_if_point_inside(client_ship, x, y, |_, _, module| {
                        let Vec2{x: module_x, y: module_y} = module.get_render_position();
                        
                        let ctx = ctx.trans(self.render_area.x + ENEMY_OFFSET_X, self.render_area.y + ENEMY_OFFSET_Y);
                            
                        for x in (0..module.shape.side()) {
                            for y in (0..module.shape.side()) {
                                if module.shape.get(x, y) == b'#' {
                                    let offset_x = x as f64 * 48.0;
                                    let offset_y = y as f64 * 48.0;
                                    
                                    Rectangle::new([0.0, 1.0, 0.0, 0.5])
                                        .draw(
                                            [module_x + offset_x, module_y + offset_y, 48.0, 48.0],
                                            &ctx.draw_state, ctx.transform,
                                            gl
                                        );
                                }
                            }
                        }
                    });
                },
                _ => { },
            }
        } else {
            // If not currently selecting a module, highlight modules the user mouses-over
            let x = self.mouse_pos.x - SHIP_OFFSET_X;
            let y = self.mouse_pos.y - SHIP_OFFSET_Y;

            apply_to_module_if_point_inside(client_ship, x, y, |_, ship_state, module| {
                let Vec2{x: module_x, y: module_y} = module.get_render_position();
            
                let ctx = ctx.trans(SHIP_OFFSET_X, SHIP_OFFSET_Y);
                
                for x in (0..module.shape.side()) {
                    for y in (0..module.shape.side()) {
                        if module.shape.get(x, y) == b'#' {
                            let offset_x = x as f64 * 48.0;
                            let offset_y = y as f64 * 48.0;
                            if self.plans.module_plans(module.index).active {
                                Rectangle::new([0.0, 0.0, 1.0, 0.5])
                                    .draw(
                                        [module_x + offset_x, module_y + offset_y, 48.0, 48.0],
                                        &ctx.draw_state, ctx.transform,
                                        gl
                                    );
                            } else if self.plans.can_plan_activate_module(ship_state, module) {
                                Rectangle::new([1.0, 1.0, 0.0, 0.5])
                                    .draw(
                                        [module_x + offset_x, module_y + offset_y, 48.0, 48.0],
                                        &ctx.draw_state, ctx.transform,
                                        gl
                                    );
                            }
                        }
                    }
                }
            });
        }*/
        
        self.chat_button.draw(ctx);
        self.star_map_button.draw(ctx);
        self.logout_button.draw(ctx);
        self.nav_map_button.draw(ctx);

        // Draw target icons
        /*for (i, icon) in self.target_icons.iter().enumerate() {
            let i = i as f64;
            
            let icon_x = 715.0+(i*100.0);
            let icon_y = 5.0;
            
            let target_screen = [840.0, 74.0, 1140.0, 104.0];
            
            let mut highlight_color = [0.0; 4];
            match self.render_area.ship {
                Some(ship) if ship == icon.ship => {
                    highlight_color = [1.0, 0.0, 0.0, 0.5];
                },
                _ => {
                    if self.mouse_pos.x >= icon_x && self.mouse_pos.x <= icon_x+96.0 &&
                        self.mouse_pos.y >= icon_y && self.mouse_pos.y <= icon_y+96.0
                    {
                        highlight_color = [0.0, 0.0, 1.0, 0.5];
                    }
                },
            }
            
            icon.draw(bc, &ctx, gl, glyph_cache, asset_store, target_screen, i, highlight_color);
        }*/

        Ok(())
    }
    
    fn on_key_pressed(&mut self, key: Keycode) {
    }
    
    fn on_mouse_left_pressed(&mut self, bc: &BattleContext, mouse_pos: Vec2f, client_ship: &Ship) {
        // Handle module plan powering and selection
        if self.selection.is_none() {
            let x = mouse_pos.x - (SHIP_OFFSET_X as f64);
            let y = mouse_pos.y - (SHIP_OFFSET_Y as f64);
            
            let mut exit_after = false;
            
            apply_to_module_if_point_inside(client_ship, x, y, |_, ship_state, module| {
                if self.plans.module_plans(module.index).active {
                    if let Some(target_mode) = module.get_target_mode() {
                        // Select this module to begin targeting
                        self.selection = Some((module.index, target_mode));
                        exit_after = true;
                    }
                } else if self.plans.can_plan_activate_module(ship_state, module) {
                    self.plans.plan_activate_module(module);
                    exit_after = true;
                }
            });
            
            if exit_after {
                return;
            }
        }
        
        let mut clear_selection = false;
        
        if let Some(ref selection) = self.selection {
            use module::{TargetData, TargetMode};
        
            let &(selected_module, ref target_mode) = selection;

            match *target_mode {
                TargetMode::TargetModule => {
                    let x = mouse_pos.x - ((self.render_area.x - ENEMY_OFFSET_X) as f64);
                    let y = mouse_pos.y - ((self.render_area.y - ENEMY_OFFSET_Y) as f64);
                    
                    if let Some(ship) = self.render_area.ship {
                        if !ship.get(bc).jumping && !ship.get(bc).exploding {
                            let ref mut plans = self.plans;
                            
                            apply_to_module_if_point_inside(ship.get(bc), x, y, |ship_index, _, module| {
                                plans.module_plans(selected_module).target =
                                    Some(module::Target {
                                        ship: ship_index,
                                        data: TargetData::TargetModule(module.index),
                                    });
                                clear_selection = true;
                            });
                        }
                    }
                },
                TargetMode::OwnModule => {
                    let x = mouse_pos.x - (SHIP_OFFSET_X as f64);
                    let y = mouse_pos.y - (SHIP_OFFSET_Y as f64);
                    
                    let ref mut plans = self.plans;
                    
                    apply_to_module_if_point_inside(client_ship, x, y, |ship_index, _, module| {
                        plans.module_plans(selected_module).target =
                            Some(module::Target {
                                ship: ship_index,
                                data: TargetData::OwnModule(module.index),
                            });
                        clear_selection = true;
                    });
                },
                TargetMode::Beam(beam_length) => {
                    let x = mouse_pos.x - ((self.render_area.x - ENEMY_OFFSET_X) as f64);
                    let y = mouse_pos.y - ((self.render_area.y - ENEMY_OFFSET_Y) as f64);
                    let beam_length = (beam_length as f64) * 48.0;
                    
                    if x >= 0.0 && y >= 0.0 {
                        if let Some(ship) = self.render_area.ship {
                            if !ship.get(bc).jumping && !ship.get(bc).exploding {
                                if let Some(beam_start) = self.beam_targeting_state {
                                    let beam_end = calculate_beam_end(beam_start, Vec2 { x: x, y: y }, beam_length);
                                    self.plans.module_plans(selected_module).target =
                                        Some(module::Target {
                                            ship: ship,
                                            data: TargetData::Beam(beam_start, beam_end),
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

            if mouse_pos.x >= icon_x && mouse_pos.x <= icon_x+icon_w &&
                mouse_pos.y >= icon_y && mouse_pos.y <= icon_y+icon_h
            {
                let mut should_change = false;

                if let Some(ship) = self.render_area.ship { // switching to a new ship
                    if ship != icon.ship {
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
    
    fn on_mouse_right_pressed(&mut self, bc: &BattleContext, mouse_pos: Vec2f, client_ship: &Ship) {
        let mut module_was_deactivated = false;
    
        if self.selection.is_none() {
            let x = mouse_pos.x - (SHIP_OFFSET_X as f64);
            let y = mouse_pos.y - (SHIP_OFFSET_Y as f64);
            
            apply_to_module_if_point_inside(client_ship, x, y, |_, ship_state, module| {
                if module.get_power() > 0 && self.plans.module_plans(module.index).active {
                    self.plans.plan_deactivate_module(module);
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
    
    pub fn on_ship_removed(&mut self, client_ship: ShipIndex, removed: ShipIndex) {
        if client_ship == removed && self.selection.is_some() {
            self.selection = None;
        }
        
        self.remove_lock(removed);
        
        self.plans.on_ship_removed(removed);
    }
    
    pub fn set_client_ship(&mut self, client_ship: &Ship) {
        self.plans = client_ship.create_plans();
    }

    pub fn set_next_waypoint(&mut self) {
        //self.plans.next_waypoint = self.nav_map_gui.get_next_waypoint();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Applies function to a module in the ship if the mouse is over the module.
/// Returns whether or not the function was applied.
pub fn apply_to_module_if_point_inside<F>(ship: &Ship, x: f64, y: f64, mut f: F)
    where
        F: FnMut(ShipIndex, &ShipState, &Module)
{
    for module in ship.modules.iter() {
        for cx in (0..module.shape.side()) {
            for cy in (0..module.shape.side()) {
                if module.shape.get(cx, cy) == b'#' {
                    // Get module position and size on screen
                    let Vec2{x: module_x, y: module_y} = module.get_render_position() + Vec2::new(cx as f64, cy as f64)*48.0;
                    if x >= module_x && x <= module_x+48.0 && y >= module_y && y <= module_y+48.0 {
                        f(ship.index, &ship.state, module);
                        return;
                    }
                }
            }
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
    fn draw(&self, bc: &BattleContext,
            ctx: &mut Context,
            asset_store: &AssetStore,
            target_screen: [f64; 4],
            i: f64,
            highlight_color: [f32; 4])
    {
        let (target_screen_x1, target_screen_h1, target_screen_x2, target_screen_h2) =
            (target_screen[0], target_screen[1], target_screen[2], target_screen[3]);
        
        let ship = self.ship.get(bc);
        
        let icon = match ship.get_height() {
            1...2 => asset_store.get_texture_str("small_target"),
            3 => asset_store.get_texture_str("medium_target"),
            4...255 => asset_store.get_texture_str("big_target"),
            _ => unreachable!(),
        };
        let icon_w = icon.width();
        let icon_h = icon.height();
        
        let section_width = (target_screen_x2 - target_screen_x1)/5.0;
        
        let icon_offset_x = (section_width - icon_w as f64) / 2.0;
        let icon_offset_y_top = (target_screen_h1 - icon_h as f64) / 2.0 - 10.0;
        let icon_offset_y_bot = (target_screen_h1 - icon_h as f64) / 2.0 + 10.0;
        
        let icon_x = target_screen_x1 + (target_screen_x2 - target_screen_x1)*(i/5.0);
        let icon_y = 2.0;
        let icon_x2 = target_screen_x1 + (target_screen_x2 - target_screen_x1)*((i + 1.0)/5.0);
        let icon_cur_h1 = target_screen_h1 + (target_screen_h2 - target_screen_h1)*(i/5.0);
        let icon_cur_h2 = target_screen_h1 + (target_screen_h2 - target_screen_h1)*((i + 1.0)/5.0);
        
        //let (half_icon_w, half_icon_h) = ((icon_w/2) as f64, (icon_h/2) as f64);
        //image(icon.deref(), ctx.trans(48.0 - half_icon_w, 34.0 - half_icon_h).transform, gl);
        
        /*gl.tri_list_uv(
            &ctx.draw_state,
            &[1.0; 4],
            icon.deref(),
            |f| f(
                &squish_rect_tri_list_xy(ctx.transform, [icon_x + icon_offset_x,
                                                             icon_y + icon_offset_y_top,
                                                             icon_x2 - icon_offset_x,
                                                             icon_cur_h1 - icon_offset_y_bot,
                                                             icon_cur_h2 - icon_offset_y_bot]),
                &triangulation::rect_tri_list_uv(icon.deref(), [0, 0, icon_w as i32, icon_h as i32])
            )
        );
        
        // Squished highlight rectangle
        gl.tri_list(
            &ctx.draw_state,
            &highlight_color,
            |f| f(
                &squish_rect_tri_list_xy(ctx.transform, [icon_x, icon_y, icon_x2, icon_cur_h1, icon_cur_h2]),
            )
        );*/
        
        // Draw stats bars
        
        let hp = ship.state.hp as f64;
        //let max_hp = ship.state.max_hp as f64;
        let shields = ship.state.shields as f64;
        let max_shields = ship.state.max_shields as f64;
        let power = ship.state.available_power() as f64;
        let max_power = ship.state.max_power as f64;
        
        // HP
        //Rectangle::new([0.0, 1.0, 0.0, 0.5])
        //    .draw([2.0, 72.0, max_hp, 3.0], &ctx.draw_state, ctx.transform, gl);
        /*Rectangle::new([0.0, 1.0, 0.0, 1.0])
            .draw([2.0, 72.0, hp, 3.0], &ctx.draw_state, ctx.transform, gl);
        
        // Shields
        Rectangle::new([0.0, 0.0, 1.0, 0.5])
            .draw([2.0, 76.0, max_shields, 3.0], &ctx.draw_state, ctx.transform, gl);
        Rectangle::new([0.0, 0.0, 1.0, 1.0])
            .draw([2.0, 76.0, shields, 3.0], &ctx.draw_state, ctx.transform, gl);
        
        // Power
        Rectangle::new([1.0, 1.0, 0.0, 0.5])
            .draw([2.0, 80.0, max_power, 3.0], &ctx.draw_state, ctx.transform, gl);
        Rectangle::new([1.0, 1.0, 0.0, 1.0])
            .draw([2.0, 80.0, power, 3.0], &ctx.draw_state, ctx.transform, gl);*/
        
        
        // Draw ship's name
        {
            /*let ctx = ctx.trans(2.0, 94.0);
            Text::new_color([1.0; 4], 10).draw(
                ship.name.as_str(),
                glyph_cache,
                &ctx.draw_state, ctx.transform,
                gl,
            );*/
        }
    }
}

/// Creates triangle list vertices from a rotated rectangle.
/*#[inline(always)]
pub fn squish_rect_tri_list_xy(m: Matrix2d, rect: [f64; 5]) -> [f32; 12] {
    use graphics::triangulation;
    
    let (x, y, x2, bot_left, bot_right) = (rect[0], rect[1], rect[2], rect[3], rect[4]);
    [
        triangulation::tx(m, x, y), triangulation::ty(m, x, y),
        triangulation::tx(m, x2, y), triangulation::ty(m, x2, y),
        triangulation::tx(m, x, bot_left), triangulation::ty(m, x, bot_left),
        triangulation::tx(m, x2, y), triangulation::ty(m, x2, y),
        triangulation::tx(m, x2, bot_right), triangulation::ty(m, x2, bot_right),
        triangulation::tx(m, x, bot_left), triangulation::ty(m, x, bot_left)
    ]
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ShipRenderArea {
    ship: Option<ShipIndex>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    //target: RenderTexture,
    //texture: Image,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn draw_ship(
    gtx: &mut ReforgeClientContext, ctx: &mut Context, sim_effects: &mut SimEffects,
    ship: &Ship, time: f64)
    -> GameResult<()>
{
    ship.draw(ctx, &gtx.asset_store);
    sim_effects.update(ctx, ship.id, time);
    
    if !ship.exploding {
        ship.draw_module_hp(ctx);
    }
    
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn draw_stats(
    ctx: &mut Context, stats_labels: &StatsLabels, plans: &ShipPlans,
    ship: &Ship, is_client_ship: bool)
    -> GameResult<()>
{
    use std::cmp;

    use graphics::*;
    
    let hp_rect = graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.0, 1.0, 0.0, 1.0));
    let shield_rect = graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.0, 0.0, 1.0, 1.0));
    let power_rect = graphics::rectangle(ctx, DrawMode::Fill, Rect::new(1.0, 1.0, 0.0, 1.0));
    
    {
        graphics::set_color(ctx, [0.0, 1.0, 0.0, 1.0].into())?;
        for i in 0..ship.state.get_hp() {
            graphics::rectangle(
                ctx, graphics::DrawMode::Fill,
                Rect::new((i as f32) * 10.0 + 5.0, 5.0 + 14.0, 8.0, 16.0))?;
        }
    }
    
    {
        graphics::set_color(ctx, [0.0, 0.0, 1.0, 1.0].into())?;
        for i in 0..ship.state.shields {
            graphics::rectangle(
                ctx, graphics::DrawMode::Fill,
                Rect::new((i as f32) * 10.0 + 5.0, 5.0 + 52.0, 8.0, 16.0))?;
        }
    }
    
    with_translate(ctx, Point2::new(5.0, 5.0 + 90.0), |ctx| -> GameResult<()> {
        if is_client_ship {
            let available_power = ship.state.available_power();
            let available_plan_power = plans.available_plan_power(&ship.state);
            
            // Available power bars
            graphics::set_color(ctx, [1.0, 1.0, 0.0, 1.0].into())?;
            for i in 0..cmp::min(available_plan_power, available_power) {
                graphics::rectangle(
                    ctx, graphics::DrawMode::Fill,
                    Rect::new((i as f32) * 10.0 + 5.0, 5.0 + 90.0, 8.0, 16.0))?;
            }
        
            if available_plan_power < available_power {
                // Used power bars
                graphics::set_color(ctx, [1.0, 1.0, 0.0, 0.5].into())?;
                for i in available_plan_power..available_power {
                    graphics::rectangle(
                        ctx, graphics::DrawMode::Fill,
                        Rect::new((i as f32) * 10.0 + 5.0, 5.0 + 90.0, 8.0, 16.0))?;
                }
            } else if available_plan_power > available_power {
                // Newly usable power bars
                graphics::set_color(ctx, [0.0, 1.0, 0.0, 0.5].into())?;
                for i in available_power..available_plan_power {
                    graphics::rectangle(
                        ctx, graphics::DrawMode::Fill,
                        Rect::new((i as f32) * 10.0 + 5.0, 5.0 + 90.0, 8.0, 16.0))?;
                }
            }
        } else {
            // Available power bars for another player's ship
            graphics::set_color(ctx, [1.0, 1.0, 0.0, 1.0].into())?;
            for i in 0..ship.state.available_power() {
                graphics::rectangle(
                    ctx, graphics::DrawMode::Fill,
                    Rect::new((i as f32) * 10.0 + 5.0, 5.0 + 90.0, 8.0, 16.0))?;
            }
        }

        // Back to white
        graphics::set_color(ctx, [1.0; 4].into())?;

        Ok(())
    })?;
    
    // Draw labels for hp, shields and power meters
    /*image(&stats_labels.hp_texture, ctx.trans(5.0, 4.0).transform, gl);
    image(&stats_labels.shield_texture, ctx.trans(5.0, 42.0).transform, gl);
    image(&stats_labels.power_texture, ctx.trans(5.0, 80.0).transform, gl);
    
    {
        let ctx = ctx.trans(5.0, 160.0);
        Text::new_color([1.0; 4], 30).draw(
            ship.name.as_str(),
            glyph_cache,
            &ctx.draw_state, ctx.transform,
            gl,
        );
    }*/

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Yo dawg imma draw me some space stars

// TODO
/*struct Star {
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
        for i in 0..stars.capacity() {
            let mut layer = Vec::with_capacity(50);
            let star_count: u32 = 
                if i == 0 {
                   1000
                } else {
                    100
                };
            let size =
                if i == 0 {
                    1.0
                } else {
                    (rng.gen::<u8>() % 5 + 1) as f64
                };
            println!("star_count: {}", star_count);
            for _ in 0..star_count {
                layer.push(Star {
                    position: [rng.gen::<f64>() * 1290.0, rng.gen::<f64>() * 730.0],
                    size: size,
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
            for star in stars.iter_mut() {
                if i == 0 {
                    star.position[0] -= 2.0*dt;
                } else {
                    star.position[0] -= (50.0/(i+1) as f64)*dt;
                }
                
                if star.position[0] < 0.0 - star.size {
                    star.position[0] += 1290.0
                }
            }
        }
    }
    
    pub fn draw(&self, ctx: &mut Context) {
        use graphics::*;
        
        let star_rect = Rectangle::new([1.0, 1.0, 1.0, 1.0]);
        for stars in self.stars.iter() {
            for star in stars.iter() {
                star_rect.draw([star.position[0], star.position[1], star.size, star.size],
                               &ctx.draw_state, ctx.transform, gl);
            }
        }
    }
}*/
