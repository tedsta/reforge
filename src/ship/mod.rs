use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;
use std::marker::Reflect;

use battle_context::BattleContext;
use module;
use module::{
    IModule,
    IModuleRef,
    IModuleStored,
    IModuleNetworked,
    Module,
    ModuleBase,
    ModuleBox,
    ModuleIndex,
    ModuleRef,
    ModuleStats,
    ModuleStoredBox,
    ModuleNetworkedBox,
    Target,
    TargetManifest,
};
use net::{ClientId, InPacket, OutPacket};
use sector_data::SectorId;
use self::ship_gen::generate_ship;
use sim::SimEvents;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use asset_store::AssetStore;
#[cfg(feature = "client")]
use space_gui::ModuleIcons;

// Use the ship_gen module privately here
mod ship_gen;

// Holds everything about the ship's damage, capabilities, etc.
#[derive(Clone, RustcEncodable, RustcDecodable)]
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
    
    pub fn available_plan_power(&self) -> u8 {
        if self.max_power > self.plan_power_use {
            self.max_power - self.plan_power_use
        } else {
            0
        }
    }
    
    pub fn can_activate_module(&self, module: &ModuleBase) -> bool {
        if module.can_activate() && self.available_power() >= module.get_power() {
            true
        } else {
            false
        }
    }
    
    pub fn can_plan_activate_module(&self, module: &ModuleBase) -> bool {
        if module.can_plan_activate() && self.available_plan_power() >= module.get_power() {
            true
        } else {
            false
        }
    }
    
    pub fn plan_activate_module(&mut self, module: &mut ModuleBase) {
        self.plan_power_use += module.get_power();
        module.plan_powered = true;
    }
    
    pub fn plan_deactivate_module(&mut self, module: &mut ModuleBase) {
        self.plan_power_use -= module.get_power();
        module.plan_powered = false;
    }
    
    fn pre_before_simulation(&mut self) {
        self.shields = 0;
    }
    
    pub fn deal_damage(&mut self, module_index: ModuleIndex, damage: u8) {
        // Can't deal more damage than there is HP
        let damage = cmp::min(self.hp, damage);
        
        if self.shields > 0 {
            self.shields -= cmp::min(self.shields, damage);
        } else {
            // Get the amount of damage dealt to the module
            let damage =
                self.module_stats
                    .get_mut(module_index.to_usize())
                    .expect("Failed to deal damage to non-existant module")
                    .deal_damage(damage);
            
            // Adjust the ship's HP state
            self.hp -= damage;
        }
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

pub type ShipRef = Rc<RefCell<Ship>>;

// Type for the ID of a ship
pub type ShipId = u64;

#[derive(Copy, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct ShipIndex(pub u32);

impl ShipIndex {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

pub struct Ship {
    pub id: ShipId,
    pub name: String,
    pub client_id: Option<ClientId>,
    pub index: ShipIndex, // Index for ship in ship vector in BattleContext
    pub state: ShipState,
    pub modules: Vec<ModuleRef>,
    
    // Ship dimensions in module blocks
    width: u8,
    height: u8,
    
    pub level: u8, // TODO: This is very temporary only for IC US semifinals
    
    // Ship's sector jumping plans
    pub target_sector: Option<SectorId>,
    
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
            
            level: level,
            
            target_sector: None,
            jumping: false,
            exploding: false,
        }
    }
    
    pub fn generate(id: ShipId, name: String, level: u8) -> Ship {
        generate_ship(id, name, level)
    }
    
    pub fn get_width(&self) -> u8 {
        self.width
    }
    
    pub fn get_height(&self) -> u8 {
        self.height
    }
    
    pub fn is_space_free(&self, x: u8, y: u8, width: u8, height: u8) -> bool {
        for module in self.modules.iter() {
            let module = (*module).borrow();
            let base = module.get_base();
            
            if base.x + base.width > x && base.x < x + width && base.y + base.height > y && base.y < y + height {
                return false;
            }
        }
        
        true
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module<M>(&mut self, mut module: Module<M>) -> bool
        where M: IModule + Reflect + Clone + 'static
    {
        // Add to state hp
        self.state.total_module_hp += module.get_base().get_hp();
        self.state.hp = self.state.total_module_hp/2;
        self.state.module_stats.push(module.get_base().stats);
        
        // Modify the ship's dimensions
        self.width = cmp::max(self.width, module.get_base().x + module.get_base().width);
        self.height = cmp::max(self.height, module.get_base().y + module.get_base().height);
        
        // Setup module's index
        module.get_base_mut().index = ModuleIndex(self.modules.len() as u32);
        
        // Activate module if can
        if module.get_base().is_active() {
            module.on_activated(&mut self.state);
        }
        
        // Add the module
        self.modules.push(Rc::new(RefCell::new(ModuleBox::new(module))));
        true
    }
    
    /// Returns a list of modules hit by the specified beam slong with the normalized time that the
    /// beam will hit each module
    pub fn beam_hits<F>(&self, start: Vec2f, end: Vec2f, mut to_apply: F)
        where
            F: FnMut(&ModuleRef, Vec2f, f64, Option<f64>)
    {
        use std::num::Float;
        use std::ops::Deref;
        
        // We are using the algorithm described here:
        // http://stackoverflow.com/a/1084899/4006804
    
        for module in &self.modules {
            let module_borrowed = module.borrow();
            let module_size = module_borrowed.get_base().get_render_size();
            
            let circle_pos = module_borrowed.get_base().get_render_center();
            let circle_radius = module_size.x.min(module_size.y) / 2.5;
            
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
        }
    }
    
    pub fn server_preprocess(&mut self, context: &BattleContext) {
        for module in self.modules.iter() {
            let target = module.borrow().get_base().target.as_ref().map(|t| TargetManifest::from_target(context, t));
            module.borrow_mut().server_preprocess(&mut self.state, target);
        }
    }
    
    pub fn before_simulation(&self, context: &BattleContext, events: &mut SimEvents) {
        for module in self.modules.iter() {
            let target = module.borrow().get_base().target.as_ref().map(|t| TargetManifest::from_target(context, t));
            module.borrow_mut().before_simulation(events, target);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_plan_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects, ship_ref: &ShipRef) {
        for module in self.modules.iter() {
            module.borrow().add_plan_effects(asset_store, effects, ship_ref);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn add_simulation_effects(&self, context: &BattleContext, asset_store: &AssetStore, effects: &mut SimEffects, ship_ref: &ShipRef) {
        if self.exploding {
            self.add_exploding_effects(asset_store, effects, ship_ref);
        } else {
            for module in self.modules.iter() {
                let target = module.borrow().get_base().target.as_ref().map(|t| TargetManifest::from_target(context, t));
                module.borrow().add_simulation_effects(asset_store, effects, ship_ref, target);
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_exploding_effects(&self, asset_store: &AssetStore, effects: &mut SimEffects, ship_ref: &ShipRef) {
        use std::rand;
        use std::rand::Rng;
    
        use sim_visuals::SpriteVisual;
        use sprite_sheet::{SpriteSheet, SpriteAnimation};
    
        for module in self.modules.iter() {
            module.borrow_mut().get_base_mut().stats.hp = 0;
            module.borrow().add_plan_effects(asset_store, effects, ship_ref);
        }
        
        // Random number generater
        let mut rng = rand::thread_rng();
    
        for _ in 0..50 {
            let x = rng.gen::<f64>() * ((self.width as f64) * 48.0);
            let y = rng.gen::<f64>() * ((self.height as f64) * 48.0);
            let time = rng.gen::<f64>() * 4.5;
        
            let mut sprite = SpriteSheet::new(asset_store.get_sprite_info_str("effects/ship_explosion1.png"));
            sprite.centered = true;
            sprite.add_animation(SpriteAnimation::PlayOnce(time, time+0.5, 0, 8));
        
            effects.add_visual(self.id, 2, box SpriteVisual {
                position: Vec2 { x: x, y: y },
                sprite_sheet: sprite,
            });
            
            effects.add_sound(time, 0, asset_store.get_sound(&"effects/ship_explosion1.ogg".to_string()).clone());
        }
    }
    
    pub fn after_simulation(&mut self) {
        for module in self.modules.iter() {
            module.borrow_mut().after_simulation(&mut self.state);
        }
    }
    
    pub fn apply_module_stats(&mut self) {
        let module_stats: Vec<ModuleStats> = self.state.module_stats.iter().cloned().collect();
        for (module, stats) in self.modules.iter().zip(module_stats.iter()) {
            let mut module = module.borrow_mut();
            
            // Get if module was active before applying the stats
            let was_active = module.get_base().is_active();
            
            if module.get_base().stats.hp != stats.hp {
                module.get_base_mut().stats.hp = stats.hp;
            }
            
            // Activate or deactivate module if the active state changed
            if was_active && !module.get_base().is_active() {
                // Module just got deactivated
                self.state.power_use -= module.get_base().get_power();
                module.get_base_mut().powered = false;
                module.on_deactivated(&mut self.state);
            }
        }
    }
    
    pub fn deactivate_unpowerable_modules(&mut self) {
        // Plan power
        for module in &self.modules {
            if self.state.plan_power_use <= self.state.max_power {
                break;
            } else {
                // Attempt to borrow the module
                if let Some(mut module) = module.try_borrow_mut() {
                    if module.get_base().get_power() > 0 {
                        if !module.get_base().powered && module.get_base().plan_powered {
                            self.state.plan_deactivate_module(module.get_base_mut());
                        }
                    }
                }
            }
        }
        
        // Power
        for module in &self.modules {
            if self.state.power_use <= self.state.max_power {
                break;
            } else {
                // Attempt to borrow the module
                if let Some(mut module) = module.try_borrow_mut() {
                    if module.get_base().get_power() > 0 {
                        if module.get_base().powered {
                            self.state.power_use -= module.get_base().get_power();
                            module.get_base_mut().powered = false;
                            module.on_deactivated(&mut self.state);
                        }
                    }
                }
            }
        }
    }
    
    pub fn on_ship_removed(&self, ship_id: ShipId) {
        for module in self.modules.iter() {
            module.borrow_mut().get_base_mut().on_ship_removed(ship_id);
        }
    }
    
    pub fn apply_module_plans(&mut self) {
        for module in self.modules.iter() {
            let mut module = module.borrow_mut();
            
            if module.get_base().plan_powered != module.get_base().powered {
                if module.get_base().plan_powered && self.state.can_activate_module(module.get_base()) {
                    module.get_base_mut().powered = true;
                    self.state.power_use += module.get_base().get_power();
                    module.on_activated(&mut self.state);
                } else if module.get_base().powered {
                    module.get_base_mut().powered = false;
                    self.state.power_use -= module.get_base().get_power();
                    module.on_deactivated(&mut self.state);
                }
                
                module.get_base_mut().plan_powered = module.get_base().powered;
            }
            
            module.get_base_mut().apply_target_plans();
        }
    }
    
    pub fn get_module_plans(&self) -> Vec<module::ModulePlans> {
        self.modules.iter().map(|m| m.borrow().get_base().get_plans()).collect()
    }
    
    pub fn set_module_plans(&self, plans: &Vec<module::ModulePlans>) {
        for (module, plans) in self.modules.iter().zip(plans.iter()) {
            module.borrow_mut().get_base_mut().set_plans(plans);
        }
    }
    
    pub fn set_targets(&self, targets: &Vec<(Option<Target>, Option<Target>)>) {
        for (module, targets) in self.modules.iter().zip(targets.iter()) {
            let &(ref target, ref plan_target) = targets;
            module.borrow_mut().get_base_mut().set_targets(target, plan_target);
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        packet.write(&self.state.power_use);
        
        // Jumping stuff
        packet.write(&self.jumping);
        
        // Modoule results
        for module in &self.modules {
            let module = module.borrow();
        
            // TODO: fix this ugliness when inheritance is a thing in Rust
            // Write the base results
            packet.write(&module.get_base().powered);
            packet.write(&module.get_base().target);
        
            module.write_results(packet);
        }
    }
    
    pub fn read_results(&mut self, context: &BattleContext, packet: &mut InPacket) {
        self.state.power_use = packet.read().ok().expect("Failed to read ShipState::power_use");
        self.jumping = packet.read().ok().expect("Failed to read Ship::jumping");
        for module in &self.modules {
            let mut module = module.borrow_mut();
            
            // TODO: fix this ugliness when inheritance is a thing in Rust
            // Read the base results
            let was_powered = module.get_base().powered;
            module.get_base_mut().powered = packet.read().ok().expect("Failed to read ModuleBase powered");
            
            if !was_powered && module.get_base().powered {
                module.on_activated(&mut self.state);
            } else if was_powered && !module.get_base().powered {
                module.on_deactivated(&mut self.state);
            }
            
            module.get_base_mut().target = packet.read().ok().expect("Failed to read ModuleBase target");
        
            module.read_results(packet);
        }
    }
    
    #[cfg(feature = "client")]
    pub fn draw(&self, context: &Context, gl: &mut Gl, asset_store: &AssetStore) {
        use std::ops::Deref;
        use graphics::*;
        
        let opacity = (self.state.shields as f32)/8.0;
    
        for module in self.modules.iter() {
            let module = module.borrow();
            let module = module.get_base();
            
            let shield_texture = asset_store.get_texture_str("effects/1_module_shield.png");
            let (shield_size_x, shield_size_y) = shield_texture.get_size();
            let (shield_size_x, shield_size_y) = (shield_size_x as f64, shield_size_y as f64);
            
            for x in module.x..module.x+module.width {
                for y in module.y..module.y+module.height {
                    let context = context.trans((x as f64) * 48.0, (y as f64) * 48.0);
                    let context = context.trans(24.0 - shield_size_x/2.0, 24.0 - shield_size_y/2.0);
                    
                    Image::colored([1.0, 1.0, 1.0, opacity])
                        .draw(shield_texture.deref(), &context.draw_state, context.transform, gl);
                }
            }
        }
    }
    
    #[cfg(feature = "client")]
    pub fn draw_module_hp(&self, context: &Context, gl: &mut Gl) {
        use quack::Set;
        use graphics::*;
    
        for (module, stats) in self.modules.iter().zip(self.state.module_stats.iter()) {
            let module = module.borrow();
            let module = module.get_base();
            
            let context = context.trans((module.x as f64) * 48.0, (module.y as f64) * 48.0);
            
            let hp_rect = Rectangle::new([0.0, 1.0, 0.0, 1.0]);
            let hp_dmg_rect = Rectangle::new([1.0, 0.0, 0.0, 0.5]);
            let armor_rect = Rectangle::new([1.0, 1.0, 0.0, 1.0]);
            let armor_dmg_rect = Rectangle::new([1.0, 1.0, 0.0, 0.5]);
        
            for i in 0..module.get_min_hp() {
                if i < stats.hp {
                    hp_rect.draw([0.0, 4.0 * (i as f64), 8.0, 2.0], &context.draw_state, context.transform, gl);
                } else {
                    hp_dmg_rect.draw([0.0, 4.0 * (i as f64), 8.0, 2.0], &context.draw_state, context.transform, gl);
                }
            }
            
            for i in module.get_min_hp()..stats.hp {
                armor_rect.draw([0.0, 4.0 * (i as f64), 8.0, 2.0], &context.draw_state, context.transform, gl);
            }
            
            for i in cmp::max(module.get_min_hp(), stats.hp)..module.get_max_hp() {
                armor_dmg_rect.draw([0.0, 4.0 * (i as f64), 8.0, 2.0], &context.draw_state, context.transform, gl);
            }
        }
    }
    
    #[cfg(feature = "client")]
    pub fn draw_module_powered_icons(&self, context: &Context, gl: &mut Gl, module_icons: &ModuleIcons) {
        use graphics::*;
    
        for module in self.modules.iter() {
            let module = module.borrow();
            let module = module.get_base();
            
            // Skip modules that aren't powerable
            if module.get_power() == 0 { continue; }
            
            // Skip modules that aren't changing powered states
            if module.plan_powered == module.powered { continue; }
            
            let context = context.trans((module.x as f64) * 48.0, (module.y as f64) * 48.0).trans((module.width as f64)*48.0 - 20.0, 2.0);
            
            if module.plan_powered {
                // Module is powering up, draw on icon
                image(&module_icons.power_on_texture, context.transform, gl);
            } else {
                // Module is powering down, draw off icon
                image(&module_icons.power_off_texture, context.transform, gl);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ShipStored {
    pub id: ShipId,
    pub name: String,
    pub state: ShipState,
    pub modules: Vec<ModuleStoredBox>,
    
    // Ship dimensions in module blocks
    width: u8,
    height: u8,
    
    pub level: u8, // TODO: This is very temporary only for IC US semifinals
    
    // Ship's sector jumping plans
    pub target_sector: Option<SectorId>,
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
            
            target_sector: None,
        }
    }
    
    pub fn from_ship(ship: Ship) -> ShipStored {
        use std::rc::try_unwrap;
    
        ShipStored {
            id: ship.id,
            name: ship.name,
            state: ship.state,
            modules: ship.modules.into_iter().map(|m| try_unwrap(m).ok().expect("Failed to unwrap Module").into_inner().to_module_stored()).collect(),
            width: ship.width,
            height: ship.height,
            level: ship.level,
            target_sector: ship.target_sector,
        }
    }
    
    pub fn to_ship(self, client_id: Option<ClientId>) -> Ship {
        Ship {
            id: self.id,
            name: self.name,
            client_id: client_id,
            index: ShipIndex(0),
            state: self.state,
            modules: self.modules.into_iter().map(|m| Rc::new(RefCell::new(m.to_module()))).collect(),
            width: self.width,
            height: self.height,
            level: self.level,
            target_sector: self.target_sector,
            jumping: false,
            exploding: false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(RustcEncodable, RustcDecodable)]
pub struct ShipNetworked {
    pub id: ShipId,
    pub name: String,
    pub client_id: Option<ClientId>,
    pub index: ShipIndex,
    pub state: ShipState,
    pub modules: Vec<ModuleNetworkedBox>,
    
    // Ship dimensions in module blocks
    width: u8,
    height: u8,
    
    pub level: u8, // TODO: This is very temporary only for IC US semifinals
    
    // Ship's sector jumping plans
    pub target_sector: Option<SectorId>,
    
    // Whether or not the ship successfully jumped
    pub jumping: bool,
    pub exploding: bool,
}

impl ShipNetworked {
    pub fn from_ship(ship: &Ship) -> ShipNetworked {
        use std::rc::try_unwrap;
    
        ShipNetworked {
            id: ship.id,
            name: ship.name.clone(),
            client_id: ship.client_id,
            index: ship.index,
            state: ship.state.clone(),
            modules: ship.modules.iter().map(|m| m.borrow().to_module_networked()).collect(),
            width: ship.width,
            height: ship.height,
            level: ship.level,
            target_sector: ship.target_sector,
            jumping: ship.jumping,
            exploding: ship.exploding,
        }
    }
    
    pub fn to_ship(self) -> (Ship, Vec<(Option<Target>, Option<Target>)>) {
        let modules: Vec<(ModuleBox, Option<Target>, Option<Target>)> =
            self.modules.into_iter().map(|m| m.to_module()).collect();
        
        let module_targets = modules.iter().map(|&(_, t, pt)| (t, pt)).collect();
        let modules = modules.into_iter().map(|(m, _, _)| Rc::new(RefCell::new(m))).collect();
    
        (Ship {
            id: self.id,
            name: self.name,
            client_id: self.client_id,
            index: self.index,
            state: self.state,
            modules: modules,
            width: self.width,
            height: self.height,
            level: self.level,
            target_sector: self.target_sector,
            jumping: self.jumping,
            exploding: self.exploding,
        }, module_targets)
    }
}

pub fn as_networked_ships(ships: &Vec<ShipRef>) -> Vec<ShipNetworked> {
    use std::ops::Deref;

    ships.iter().map(|s| ShipNetworked::from_ship(s.borrow().deref())).collect()
}
