use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{IoResult, IoError, InvalidInput};

use battle_state::BattleContext;
use net::{ClientId, InPacket, OutPacket};
use render::{Renderer, RenderTarget, TextureId};
use ship::{Ship, ShipIndex};
use sim_element::SimElement;
use vec::{Vec2, Vec2f};

// Use+reexport all of the modules
pub use self::engine::EngineModule;
pub use self::proj_weapon::ProjectileWeaponModule;

pub mod engine;
pub mod proj_weapon;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Encodable, Decodable)]
pub enum ModuleCategory {
    Weapon = 0,
    Propulsion,
}

pub struct ModuleCategoryData {
    pub name: &'static str,
    pub id: ModuleCategory,
}

pub static MODULE_CATEGORIES: [ModuleCategoryData, .. 2] = [
    ModuleCategoryData{name: "Weapon", id: Weapon},
    ModuleCategoryData{name: "Propulsion", id: Propulsion},
];

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleRef = Rc<RefCell<Module>>;

#[deriving(Encodable, Decodable, Show)]
pub struct ModuleIndex {
    pub index: u8,
    pub ship: ShipIndex,
}

#[deriving(Encodable, Decodable)]
pub enum Module {
    Engine(EngineModule),
    ProjectileWeapon(ProjectileWeaponModule),
}

impl Module {
    pub fn get_base<'a>(&'a self) -> &'a ModuleBase {
        match (*self) {
            Engine(ref m) => &m.base,
            ProjectileWeapon(ref m) => &m.base,
        }
    }
    
    pub fn get_base_mut<'a>(&'a mut self) -> &'a mut ModuleBase {
        match (*self) {
            Engine(ref mut m) => &mut m.base,
            ProjectileWeapon(ref mut m) => &mut m.base,
        }
    }
}

impl SimElement for Module {
    fn server_preprocess(&mut self, context: &BattleContext) {
        match *self {
            Engine(ref mut m) => m.server_preprocess(context),
            ProjectileWeapon(ref mut m) => m.server_preprocess(context),
        }
    }
    
    fn before_simulation(&mut self, context: &BattleContext) {
        match *self {
            Engine(ref mut m) => m.before_simulation(context),
            ProjectileWeapon(ref mut m) => m.before_simulation(context),
        }
    }
    
    fn on_simulation_time(&mut self, context: &BattleContext, tick: u32) {
        match *self {
            Engine(ref mut m) => m.on_simulation_time(context, tick),
            ProjectileWeapon(ref mut m) => m.on_simulation_time(context, tick),
        }
    }
    
    fn after_simulation(&mut self, context: &BattleContext) {
        match *self {
            Engine(ref mut m) => m.after_simulation(context),
            ProjectileWeapon(ref mut m) => m.after_simulation(context),
        }
    }
    
    fn get_critical_times(&self) -> Vec<u32> {
        match *self {
            Engine(ref m) => m.get_critical_times(),
            ProjectileWeapon(ref m) => m.get_critical_times(),
        }
    }
    
    fn draw(&mut self, renderer: &mut Renderer, context: &BattleContext, simulating: bool, time: f32) {
        match *self {
            Engine(ref mut m) => m.draw(renderer, context, simulating, time),
            ProjectileWeapon(ref mut m) => m.draw(renderer, context, simulating, time),
        }
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
        match *self {
            Engine(ref m) => m.write_plans(packet),
            ProjectileWeapon(ref m) => m.write_plans(packet),
        }
    }
    
    fn read_plans(&self, packet: &mut InPacket) {
        match *self {
            Engine(ref m) => m.read_plans(packet),
            ProjectileWeapon(ref m) => m.read_plans(packet),
        }
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
        match *self {
            Engine(ref m) => m.write_results(packet),
            ProjectileWeapon(ref m) => m.write_results(packet),
        }
    }
    
    fn read_results(&self, packet: &mut InPacket) {
        match *self {
            Engine(ref m) => m.read_results(packet),
            ProjectileWeapon(ref m) => m.read_results(packet),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Encodable, Decodable)]
pub struct ModuleBase {
    // Ship's index of this module, if it belongs to a ship
    pub index: Option<ModuleIndex>,

    // Module position/size stuff
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,

    // Module stats
    pub power: u32,
    pub max_power: u32,
    pub damage: u32,
    pub hull: u32,
    
    // Category of this module
    pub category: ModuleCategory,
    
    // Module rendering stuff
    pub texture: TextureId,
}

impl ModuleBase {
    pub fn new(category: ModuleCategory, texture: TextureId) -> ModuleBase {
        ModuleBase {
            index: None,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            power: 0,
            max_power: 1,
            damage: 0,
            hull: 0,
            category: category,
            texture: texture,
        }
    }
    
    pub fn get_ship<'a>(&self, context: &'a BattleContext) -> &'a Ship {
        match self.index {
            Some(index) => {
                match context.get_ship(&index.ship) {
                    Some(ship) => ship,
                    None => fail!("Failed to get ship {}", index),
                }
            },
            None => fail!("Cannot draw module with no ship"),
        }
    }
    
    pub fn draw(&self, renderer: &Renderer, ship: &Ship) {
        ship.render_target.draw_texture_vec(renderer, self.texture, &self.get_render_position());
    }
    
    pub fn get_render_position(&self) -> Vec2f {
        Vec2{x: (self.x as f32)*(48f32), y: (self.y as f32)*(48f32)}
    }
    
    pub fn get_render_size(&self) -> Vec2f {
        Vec2{x: (self.width as f32)*(48f32), y: (self.height as f32)*(48f32)}
    }
    
    pub fn get_render_center(&self) -> Vec2f {
        self.get_render_position() + (self.get_render_size()/2f32)
    }
}