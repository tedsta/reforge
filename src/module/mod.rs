use std::rc::Rc;
use std::cell::RefCell;

use assets::TextureId;
use net::{InPacket, OutPacket};
use ship::{ShipId, ShipState};
use sim::{SimEventAdder, SimEvents};
use vec::{Vec2, Vec2f};

#[cfg(client)]
use sfml_renderer::SfmlRenderer;
#[cfg(client)]
use sim::SimVisuals;

// Use+reexport all of the modules
pub use self::engine::EngineModule;
pub use self::proj_weapon::ProjectileWeaponModule;

pub mod engine;
pub mod proj_weapon;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Encodable, Decodable, PartialEq)]
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

pub trait IModule {
    fn server_preprocess(&mut self, ship_state: &mut ShipState);

    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder);
    #[cfg(client)]
    fn add_sim_visuals(&self, ship_id: ShipId, visuals: &mut SimVisuals);
    fn after_simulation(&mut self, ship_state: &mut ShipState);

    fn write_plans(&self, packet: &mut OutPacket);
    fn read_plans(&mut self, packet: &mut InPacket);
    
    fn write_results(&self, packet: &mut OutPacket);
    fn read_results(&mut self, packet: &mut InPacket);
    
    ////////////////////
    // GUI stuff
    
    fn on_icon_clicked(&mut self) -> bool;
    fn on_module_clicked(&mut self, ship_id: ShipId, module: &ModuleRef) -> bool;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleRef = Rc<RefCell<Module>>;

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

impl IModule for Module {
    fn server_preprocess(&mut self, ship_state: &mut ShipState) {
        match *self {
            Engine(ref mut m) => m.server_preprocess(ship_state),
            ProjectileWeapon(ref mut m) => m.server_preprocess(ship_state),
        }
    }
    
    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder) {
        match *self {
            Engine(ref mut m) => m.before_simulation(ship_state, events),
            ProjectileWeapon(ref mut m) => m.before_simulation(ship_state, events),
        }
    }
    
    #[cfg(client)]
    fn add_sim_visuals(&self, ship_id: ShipId, visuals: &mut SimVisuals) {
        match *self {
            Engine(ref m) => m.add_sim_visuals(ship_id, visuals),
            ProjectileWeapon(ref m) => m.add_sim_visuals(ship_id, visuals),
        }
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
        match *self {
            Engine(ref mut m) => m.after_simulation(ship_state),
            ProjectileWeapon(ref mut m) => m.after_simulation(ship_state),
        }
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
        match *self {
            Engine(ref m) => m.write_plans(packet),
            ProjectileWeapon(ref m) => m.write_plans(packet),
        }
    }
    
    fn read_plans(&mut self, packet: &mut InPacket) {
        match *self {
            Engine(ref mut m) => m.read_plans(packet),
            ProjectileWeapon(ref mut m) => m.read_plans(packet),
        }
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
        match *self {
            Engine(ref m) => m.write_results(packet),
            ProjectileWeapon(ref m) => m.write_results(packet),
        }
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
        match *self {
            Engine(ref mut m) => m.read_results(packet),
            ProjectileWeapon(ref mut m) => m.read_results(packet),
        }
    }
    
    fn on_icon_clicked(&mut self) -> bool {
        match *self {
            Engine(ref mut m) => m.on_icon_clicked(),
            ProjectileWeapon(ref mut m) => m.on_icon_clicked(),
        }
    }
    
    fn on_module_clicked(&mut self, ship_id: ShipId, module: &ModuleRef) -> bool {
        match *self {
            Engine(ref mut m) => m.on_module_clicked(ship_id, module),
            ProjectileWeapon(ref mut m) => m.on_module_clicked(ship_id, module),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Encodable, Decodable)]
pub struct ModuleBase {
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
    
    // Module texture ID
    texture: TextureId,
}

impl ModuleBase {
    pub fn new(category: ModuleCategory, texture: TextureId) -> ModuleBase {
        ModuleBase {
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
    
    #[cfg(client)]
    pub fn draw(&self, renderer: &SfmlRenderer) {
        renderer.draw_texture_vec(self.texture, &self.get_render_position());
    }
    
    pub fn get_render_position(&self) -> Vec2f {
        Vec2{x: (self.x as f32)*(48f32), y: (self.y as f32)*(48f32)}
    }
    
    pub fn get_render_size(&self) -> Vec2f {
        Vec2{x: (self.width as f32)*(48f32), y: (self.height as f32)*(48f32)}
    }
    
    pub fn get_render_center(&self) -> Vec2f {
        self.get_render_position() + (self.get_render_size()/2.0)
    }
}