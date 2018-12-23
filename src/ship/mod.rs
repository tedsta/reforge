use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;
use std::collections::VecDeque;
//use std::marker::Reflect;

use battle_context::BattleContext;
use module;
use module::{
    ModelStore,
    IModule,
    Module,
    ModuleIndex,
    ModuleShape,
    ModuleStats,
    ModuleStored,
    Target,
    TargetManifest,
};
use net::{ClientId, InPacket, OutPacket};
use self::ship_gen::{generate_ship, generate_dummy_ship, generate_dev_ship};
use sim::SimEvents;
use util::with_translate;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use ggez::{
    Context, GameResult,
    graphics::{self, DrawParam, DrawMode, FontId, Image, Point2, Rect, Scale, TextCached},
};

#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use asset_store::AssetStore;
#[cfg(feature = "client")]
use space_gui::ModuleIcons;

pub use self::plans::ShipPlans;

mod ship_gen;
mod plans;

// Holds everything about the ship's damage, capabilities, etc.
#[derive(Clone, Serialize, Deserialize)]
pub struct ShipState {
    pub hp: u8,
    total_module_hp: u8, // Sum of HP of all modules, used to recalculate HP when damaged
    pub power_use: u8,
    pub plan_power_use: u8, // Keeps track of power for planning
    pub max_power: u8,
    pub thrust: u8,
    pub shields: u8,
    pub max_shields: u8,
    
    pub module_stats: Vec<ModuleStats>,
}

impl ShipState {
    pub fn new() -> ShipState {
        ShipState {
            hp: 0,
            total_module_hp: 0,
            power_use: 0,
            plan_power_use: 0,
            max_power: 0,
            thrust: 0,
            shields: 0,
            max_shields: 0,
            
            module_stats: vec!(),
        }
    }
    
    pub fn available_power(&self) -> u8 {
        if self.max_power > self.power_use {
            self.max_power - self.power_use
        } else {
            0
        }
    }
    
    pub fn can_activate_module(&self, module: &Module) -> bool {
        if module.can_activate() && self.available_power() >= module.get_power() {
            true
        } else {
            false
        }
    }
    
    pub fn deal_damage(&mut self,
                       module_index: ModuleIndex,
                       damage: u8,
                       shield_piercing: u8,
                       damage_shields: bool) {
        let shield_absorption =
            if self.shields > shield_piercing {
                cmp::min(self.shields - shield_piercing, damage)
            } else {
                0
            };
        
        let ship_damage = damage - shield_absorption;
        
        if damage_shields {
            self.shields -= shield_absorption;
        }
        
        // Get the amount of damage dealt to the module
        self.module_stats
            .get_mut(module_index.to_usize())
            .expect("Failed to deal damage to non-existant module")
            .deal_damage(ship_damage);
        
        // Adjust the ship's HP state
        self.hp -= cmp::min(self.hp, ship_damage);
    }
    
    pub fn repair_damage(&mut self, module_index: ModuleIndex, repair: u8) {
        // Get the amount of damage dealt to the module
        let repair =
            self.module_stats
                .get_mut(module_index.to_usize())
                .expect("Failed to deal damage to non-existant module")
                .repair_damage(repair);
    }
    
    pub fn add_power(&mut self, power: u8) {
        self.max_power += power;
    }
    
    pub fn remove_power(&mut self, power: u8) {        
        self.max_power -= power;
    }
    
    pub fn return_power(&mut self, power: u8) {
        self.power_use -= power;
        self.plan_power_use -= power;
    }
    
    pub fn add_shields(&mut self, shields: u8) {
        self.max_shields += shields;
    }
    
    pub fn remove_shields(&mut self, shields: u8) {
        self.max_shields -= shields;
        self.shields = cmp::min(self.shields, self.max_shields);
    }
    
    pub fn get_hp(&self) -> u8 {
        self.hp
    }
}

// Type for the ID of a ship
pub type ShipId = u64;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipIndex(pub u32);

impl ShipIndex {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
    
    pub fn get<'a>(&self, bc: &'a BattleContext) -> &'a Ship {
        bc.ships[self.0 as usize]
            .as_ref()
            .expect("Tried to access ship at empty index.")
    }
    
    pub fn get_mut<'a>(&self, bc: &'a mut BattleContext) -> &'a mut Ship {
        bc.ships.get_mut(self.0 as usize)
            .expect("Tried to mutably access ship at invalid index.")
            .as_mut()
            .expect("Tried to mutably access ship at empty index.")
    }
}

#[derive(Serialize, Deserialize)]
pub struct Ship {
    pub id: ShipId,
    pub name: String,
    pub client_id: Option<ClientId>,
    pub index: ShipIndex, // Index for ship in ship vector in BattleContext
    pub state: ShipState,
    pub modules: Vec<Module>,
    
    // Ship dimensions in module blocks
    width: u8,
    height: u8,
    
    // Nav map stuff
    pub position: Vec2f,
    pub next_waypoint: Option<Vec2f>,
    
    pub level: u8, // TODO: This is very temporary only for IC US semifinals
    
    // Whether or not the ship successfully jumped
    pub jumping: bool,
    
    pub exploding: bool,
}

impl Ship {
    pub fn new(id: ShipId, name: String, level: u8) -> Ship {
        Ship {
            id: id,
            name: name,
            client_id: None,
            index: ShipIndex(0),
            state: ShipState::new(),
            modules: vec!(),
            
            width: 0,
            height: 0,
            
            position: Vec2::new(0.0, 0.0),
            next_waypoint: None,
            
            level: level,

            jumping: false,
            exploding: false,
        }
    }
    
    pub fn generate(id: ShipId, name: String, level: u8) -> Ship {
        generate_ship(id, name, level)
    }
    
    pub fn generate_dummy(id: ShipId, name: String) -> Ship {
        generate_dummy_ship(id, name)
    }
    
    pub fn generate_dev(model_store: &ModelStore, id: ShipId, name: String) -> Ship {
        generate_dev_ship(model_store, id, name)
    }
    
    pub fn get_width(&self) -> u8 {
        self.width
    }
    
    pub fn get_height(&self) -> u8 {
        self.height
    }

    pub fn lerp_next_waypoint(&self, time: f64) -> Vec2f {
        if let Some(next_waypoint) = self.next_waypoint {
            self.position + (next_waypoint - self.position)*(time/5.0)
        } else {
            self.position
        }
    }
    
    pub fn is_space_free(&self, x: u8, y: u8, shape: &ModuleShape) -> bool {
        for module in &self.modules {
            if module.x + module.shape.side() > x && module.x < x + shape.side() && module.y + module.shape.side() > y && module.y < y + shape.side() {
                let (start_x, start_y, end_x, end_y) =
                    (cmp::max(x, module.x),
                     cmp::max(y, module.y),
                     cmp::min(x + shape.side(), module.x + module.shape.side()),
                     cmp::min(y + shape.side(), module.y + module.shape.side()));
                
                for cx in (start_x..end_x) {
                    for cy in (start_y..end_y) {
                        if shape.get(cx - x, cy - y) == b'#' &&
                            module.shape.get(cx - module.x, cy - module.y) == b'#' {
                            return false;
                        }
                    }
                }
            }
        }
        
        true
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module(&mut self, mut module: Module) -> bool {
        // Add to state hp
        self.state.total_module_hp += module.get_hp();
        self.state.hp = self.state.total_module_hp/2;
        self.state.module_stats.push(module.stats);
        
        // Modify the ship's dimensions
        self.width = cmp::max(self.width, module.x + module.shape.side());
        self.height = cmp::max(self.height, module.y + module.shape.side());
        
        // Setup module's index
        module.index = ModuleIndex(self.modules.len() as u32);
        
        // Activate module if can
        if module.get_power() == 0 && !module.is_damaged() {
            module.active = true;
            module.inner.borrow_mut().on_activated(&mut self.state);
        }
        
        // Add the module
        self.modules.push(module);
        true
    }
    
    /// Returns a list of modules hit by the specified beam slong with the normalized time that the
    /// beam will hit each module
    pub fn beam_hits<F>(&self, beam: Option<(Vec2f, Vec2f)>, mut to_apply: F)
        where
            F: FnMut(&Module, Vec2f, f64, Option<f64>)
    {
        use num::Float;
        use std::ops::Deref;
    
        for module in &self.modules {
            let module_size = module.get_render_size();
            
            for x in (0..module.shape.side()) {
                for y in (0..module.shape.side()) {
                    if module.shape.get(x, y) == b'#' {
                        let circle_pos = module.get_render_position() + (Vec2::new(x as f64, y as f64)*48.0 + Vec2::new(48.0/2.0, 48.0/2.0));
                        let circle_radius = 48.0 / 2.5;
                        
                        match beam {
                            Some((start, end)) => {
                                // We are using the algorithm described here:
                                // http://stackoverflow.com/a/1084899/4006804
                            
                                // The beam's direction vector
                                let d = end - start;
                                
                                // The vector from the circle center to the beam start
                                let f = start - circle_pos;
                                
                                // Some variables for the algorithm. These correspond to variables in the quadratic
                                // formula.
                                let a = d.dot(d);
                                let b = 2.0 * f.dot(d);
                                let c = f.dot(f) - circle_radius*circle_radius;
                                
                                let discriminant = b*b - 4.0*a*c;
                                
                                if discriminant < 0.0 {
                                    // No intersection
                                    to_apply(module, circle_pos, circle_radius, None);
                                } else {
                                    // Ray didn't totally miss sphere, so there is a solution to the equation.

                                    let discriminant = discriminant.sqrt();

                                    // Either solution may be on or off the ray so need to test both t1 is always the
                                    // smaller value, because BOTH discriminant and a are nonnegative.
                                    let t1 = (-b - discriminant)/(2.0*a);
                                    let t2 = (-b + discriminant)/(2.0*a);

                                    // 3x HIT cases:
                                    //          -o->             --|-->  |            |  --|->
                                    // Impale(t1 hit,t2 hit), Poke(t1 hit,t2>1), ExitWound(t1<0, t2 hit), 

                                    // 3x MISS cases:
                                    //       ->  o                     o ->              | -> |
                                    // FallShort (t1>1,t2>1), Past (t1<0,t2<0), CompletelyInside(t1<0, t2>1)

                                    if t1 >= 0.0 && t1 <= 1.0 {
                                        // Impale, poke
                                        to_apply(module, circle_pos, circle_radius, Some(t1));
                                    } else if t2 >= 0.0 && t2 <= 1.0 {
                                        // Exit wound
                                        to_apply(module, circle_pos, circle_radius, Some(t2));
                                    } else if t1 < 0.0 && t2 > 1.0 {
                                        // Completely inside
                                        to_apply(module, circle_pos, circle_radius, Some(0.0));
                                    } else {
                                        // No hit
                                        to_apply(module, circle_pos, circle_radius, None);
                                    }
                                }
                            },
                            None => {
                                to_apply(module, circle_pos, circle_radius, None);
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn server_preprocess(&self, bc: &BattleContext, model_store: &ModelStore) {
        for module in &self.modules {
            if module.active {
                let ref module_context = module.create_module_context(bc, model_store, self);
                module.inner.borrow_mut().server_preprocess(module_context);
            }
        }
    }
    
    pub fn before_simulation(&self, bc: &BattleContext, model_store: &ModelStore, events: &mut SimEvents) {
        for module in &self.modules {
            if module.active {
                let ref module_context = module.create_module_context(bc, model_store, self);
                module.inner.borrow_mut().before_simulation(module_context, events);
            }
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_plan_effects(&self, bc: &BattleContext, asset_store: &AssetStore, model_store: &ModelStore, effects: &mut SimEffects) {
        for module in &self.modules {
            let ref module_context = module.create_module_context(bc, model_store, self);
            module.inner.borrow().add_plan_effects(module_context, asset_store, effects);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_simulation_effects(&self, bc: &BattleContext, asset_store: &AssetStore, model_store: &ModelStore, effects: &mut SimEffects) {
        if self.exploding {
            self.add_exploding_effects(bc, asset_store, model_store, effects);
        } else {
            for module in &self.modules {
                let ref module_context = module.create_module_context(bc, model_store, self);
                module.inner.borrow().add_simulation_effects(module_context, asset_store, effects);
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_exploding_effects(&self, bc: &BattleContext, asset_store: &AssetStore, model_store: &ModelStore, effects: &mut SimEffects) {
        use rand;
        use rand::Rng;
    
        use sim_visuals::SpriteVisual;
        use sprite_sheet::{SpriteSheet, SpriteAnimation};
    
        for module in &self.modules {
            let ref module_context = module.create_module_context(bc, model_store, self);
            module.inner.borrow().add_plan_effects(module_context, asset_store, effects);
        }
        
        // Random number generater
        let mut rng = rand::thread_rng();
    
        for _ in 0..50 {
            let x = rng.gen::<f64>() * ((self.width as f64) * 48.0);
            let y = rng.gen::<f64>() * ((self.height as f64) * 48.0);
            let time = rng.gen::<f64>() * 4.5;
        
            let mut sprite = SpriteSheet::new(asset_store.get_sprite_info_str("ship_explosion1"));
            sprite.center();
            sprite.add_animation(SpriteAnimation::PlayOnce(time, time+0.5, 0, 8));
        
            effects.add_visual(self.id, 2, SpriteVisual::new(Vec2 { x: x, y: y }, 0.0, sprite));
            
            effects.add_sound(time, 0, asset_store.get_sound(&"effects/ship_explosion1.ogg".to_string()).clone());
        }
    }
    
    pub fn after_simulation(&mut self) {
        for module in &self.modules {
            if module.active {
                module.inner.borrow_mut().after_simulation(&mut self.state);
            }
        }
    }
    
    pub fn apply_module_stats(&mut self) {
        let module_stats: Vec<ModuleStats> = self.state.module_stats.iter().cloned().collect();
        for (module, stats) in self.modules.iter_mut().zip(module_stats.iter()) {
            if module.stats.hp != stats.hp {
                module.stats.hp = stats.hp;
            }
            
            // Activate or deactivate module if the active state changed
            if module.active && module.is_damaged() {
                // Module just got deactivated
                self.state.power_use -= module.get_power();
                module.active = false;
                module.inner.borrow_mut().on_deactivated(&mut self.state);
            } else if !module.active && module.get_power() == 0 && !module.is_damaged() {
                // Module should be re-activated
                module.active = true;
                module.inner.borrow_mut().on_activated(&mut self.state);
            }
        }
    }
    
    pub fn deactivate_unpowerable_modules(&mut self) {
        for module in &mut self.modules {
            if self.state.power_use <= self.state.max_power {
                break;
            } else {
                if module.get_power() > 0 {
                    if module.active {
                        self.state.power_use -= module.get_power();
                        module.active = false;
                        module.inner.borrow_mut().on_deactivated(&mut self.state);
                    }
                }
            }
        }
    }
    
    pub fn on_ship_removed(&mut self, ship_index: ShipIndex) {
        for module in &mut self.modules {
            module.on_ship_removed(ship_index);
        }
    }
    
    pub fn create_plans(&self) -> ShipPlans {
        ShipPlans {
            logout: false,
            target_sector: None,
            module_plans: self.modules.iter().map(|m| m.create_plans()).collect(),
            plan_power_use: self.state.power_use,
            next_waypoint: self.next_waypoint,
        }
    }
    
    pub fn apply_plans(&mut self, plans: &ShipPlans) {
        for (module, module_plans) in self.modules.iter_mut().zip(plans.module_plans.iter()) {
            // Apply powered plans
            if module_plans.active != module.active {
                if module_plans.active && self.state.can_activate_module(module) {
                    module.active = true;
                    self.state.power_use += module.get_power();
                    module.inner.borrow_mut().on_activated(&mut self.state);
                } else if module.active {
                    module.active = false;
                    self.state.power_use -= module.get_power();
                    module.inner.borrow_mut().on_deactivated(&mut self.state);
                }
            }
            
            // Apply target plans
            module.target = module_plans.target;
        }

        if let Some(next_waypoint) = self.next_waypoint {
            self.position = next_waypoint;
        }
        self.next_waypoint = plans.next_waypoint;
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        packet.write(&self.state.power_use);
        
        // Jumping stuff
        packet.write(&self.jumping);

        // Waypoint stuff
        packet.write(&self.position);
        packet.write(&self.next_waypoint);

        // Modoule results
        for module in &self.modules {
            // TODO: fix this ugliness when inheritance is a thing in Rust
            // Write the base results
            packet.write(&module.active);
            packet.write(&module.target);

            module.inner.borrow().write_results(packet);
        }
    }
    
    pub fn read_results(&mut self, packet: &mut InPacket) {
        self.state.power_use = packet.read().ok().expect("Failed to read ShipState::power_use");
        self.jumping = packet.read().ok().expect("Failed to read Ship::jumping");
        self.position = packet.read().ok().expect("Failed to read Ship::position");
        self.next_waypoint = packet.read().ok().expect("Failed to read Ship::next_waypoint");
        for module in &mut self.modules {
            // TODO: fix this ugliness when inheritance is a thing in Rust
            // Read the base results
            let was_active = module.active;
            module.active = packet.read().ok().expect("Failed to read Module powered");
            
            if !was_active && module.active {
                module.inner.borrow_mut().on_activated(&mut self.state);
            } else if was_active && !module.active {
                module.inner.borrow_mut().on_deactivated(&mut self.state);
            }

            module.target = packet.read().ok().expect("Failed to read Module target");
            
            module.inner.borrow_mut().read_results(packet);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn draw(&self, ctx: &mut Context, asset_store: &AssetStore) {
        let opacity = (self.state.shields as f32) / 8.0;
    
        for module in &self.modules {
            let shield_texture = asset_store.get_texture_str("1_module_shield");
            let shield_width = shield_texture.width();
            let shield_height = shield_texture.height();
            let (shield_width, shield_height) = (shield_width as f32, shield_height as f32);
            
            for x in module.x..module.x+module.shape.side() {
                for y in module.y..module.y+module.shape.side() {
                    if module.shape.get(x - module.x, y - module.y) == b'#' {
                        graphics::set_color(ctx, [1.0, 1.0, 1.0, opacity].into());
                        graphics::draw_ex(ctx, &**shield_texture, DrawParam {
                            dest: Point2::new(
                                (x as f32) * 48.0 + 24.0 - shield_width / 2.0,
                                (y as f32) * 48.0 + 24.0 - shield_height / 2.0,
                            ), ..Default::default()
                        });
                        graphics::set_color(ctx, [1.0, 1.0, 1.0, 1.0].into());
                    }
                }
            }
        }
    }
    
    #[cfg(feature = "client")]
    pub fn draw_module_hp(&self, ctx: &mut Context) {
        for (module, stats) in self.modules.iter().zip(self.state.module_stats.iter()) {
            with_translate(
                ctx, Point2::new((module.x as f32) * 48.0, (module.y as f32) * 48.0),
                |ctx| -> () {
                    for i in 0..module.get_min_hp() {
                        if i < stats.hp {
                            // HP
                            graphics::set_color(ctx, [0.0, 1.0, 0.0, 1.0].into());
                            graphics::rectangle(
                                ctx, DrawMode::Fill,
                                Rect::new(0.0, 4.0 * (i as f32), 8.0, 2.0));
                        } else {
                            // HP damaged
                            graphics::set_color(ctx, [1.0, 0.0, 0.0, 0.5].into());
                            graphics::rectangle(
                                ctx, DrawMode::Fill,
                                Rect::new(0.0, 4.0 * (i as f32), 8.0, 2.0));
                        }
                    }
                    
                    for i in module.get_min_hp()..stats.hp {
                        // Armor
                        graphics::set_color(ctx, [1.0, 1.0, 0.0, 1.0].into());
                        graphics::rectangle(
                            ctx, DrawMode::Fill,
                            Rect::new(0.0, 4.0 * (i as f32), 8.0, 2.0));
                    }
                    
                    for i in cmp::max(module.get_min_hp(), stats.hp)..module.get_max_hp() {
                        // Armor damaged
                        graphics::set_color(ctx, [1.0, 1.0, 0.0, 0.5].into());
                        graphics::rectangle(
                            ctx, DrawMode::Fill,
                            Rect::new(0.0, 4.0 * (i as f32), 8.0, 2.0));
                    }
                });
        }

        // Reset color back to white
        graphics::set_color(ctx, [1.0, 1.0, 1.0, 1.0].into());
    }
    
    #[cfg(feature = "client")]
    pub fn draw_module_powered_icons(
        &self, ctx: &mut Context, module_icons: &ModuleIcons, plans: &ShipPlans)
        -> GameResult<()>
    {
        use graphics::*;
    
        for (module, plans) in self.modules.iter().zip(plans.module_plans.iter()) {
            // Skip modules that aren't powerable
            if module.get_power() == 0 { continue; }
            
            // Skip modules that aren't changing powered states
            if plans.active == module.active { continue; }
            
            let translate = Point2::new(
                (module.x as f32) * 48.0 + 48.0 / 2.0 - 20.0,
                (module.y as f32) * 48.0 + 2.0);
            with_translate(ctx, translate, |ctx| { 
                if plans.active {
                    // Module is powering up, draw on icon
                    graphics::draw_ex(ctx, &module_icons.power_on_texture, Default::default())
                } else {
                    // Module is powering down, draw off icon
                    graphics::draw_ex(ctx, &module_icons.power_off_texture, Default::default())
                }
            })?;
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct ShipStored {
    pub id: ShipId,
    pub name: String,
    pub state: ShipState,
    pub modules: Vec<ModuleStored>,
    
    // Ship dimensions in module blocks
    width: u8,
    height: u8,
    
    pub level: u8, // TODO: This is very temporary only for IC US semifinals
}

impl ShipStored {
    pub fn new(id: ShipId, level: u8) -> ShipStored {
        ShipStored {
            id: id,
            name: String::new(),
            state: ShipState::new(),
            modules: vec!(),
            
            width: 0,
            height: 0,
            
            level: level,
        }
    }
    
    pub fn from_ship(ship: Ship) -> ShipStored {
        ShipStored {
            id: ship.id,
            name: ship.name,
            state: ship.state,
            modules: ship.modules.into_iter().map(|m| ModuleStored::from_module(m)).collect(),
            width: ship.width,
            height: ship.height,
            level: ship.level,
        }
    }
    
    pub fn to_ship(self, client_id: Option<ClientId>) -> Ship {
        Ship {
            id: self.id,
            name: self.name,
            client_id: client_id,
            index: ShipIndex(0),
            state: self.state,
            modules: self.modules.into_iter().map(|m| m.to_module()).collect(),
            width: self.width,
            height: self.height,
            position: Vec2::new(0.0, 0.0),
            next_waypoint: None,
            level: self.level,
            jumping: false,
            exploding: false,
        }
    }
    
    pub fn get_width(&self) -> u8 {
        self.width
    }
    
    pub fn get_height(&self) -> u8 {
        self.height
    }
    
    pub fn is_space_free(&self, x: u8, y: u8, shape: &ModuleShape) -> bool {
        for module in &self.modules {
            if module.x + module.shape.side() > x && module.x < x + shape.side() && module.y + module.shape.side() > y && module.y < y + shape.side() {
                let (start_x, start_y, end_x, end_y) =
                    (cmp::max(x, module.x),
                     cmp::max(y, module.y),
                     cmp::min(x + shape.side(), module.x + module.shape.side()),
                     cmp::min(y + shape.side(), module.y + module.shape.side()));
                
                for cx in (start_x..end_x) {
                    for cy in (start_y..end_y) {
                        if shape.get(cx - x, cy - y) == b'#' &&
                            module.shape.get(cx - module.x, cy - module.y) == b'#' {
                            return false;
                        }
                    }
                }
            }
        }
        
        true
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module(&mut self, mut module: ModuleStored) -> bool {
        // Add to state hp
        self.state.total_module_hp += module.get_hp();
        self.state.hp = self.state.total_module_hp/2;
        self.state.module_stats.push(module.stats);
        
        // Modify the ship's dimensions
        self.width = cmp::max(self.width, module.x + module.shape.side());
        self.height = cmp::max(self.height, module.y + module.shape.side());
        
        // Setup module's index
        module.index = ModuleIndex(self.modules.len() as u32);
        
        // Activate module if can
        if module.get_power() == 0 && !module.is_damaged() {
            module.active = true;
            module.inner.borrow_mut().on_activated(&mut self.state);
        }
        
        // Add the module
        self.modules.push(module);
        true
    }
    
    #[cfg(feature = "client")]
    pub fn add_simulation_effects(&self, asset_store: &AssetStore, model_store: &ModelStore, effects: &mut SimEffects) {
        for module in &self.modules {
            let ref module_context = module.create_module_context(model_store, self);
            module.inner.borrow().add_simulation_effects(module_context, asset_store, effects);
        }
    }
}
