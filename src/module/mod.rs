use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{IoResult, IoError, InvalidInput};

use net::{ClientId, InPacket, OutPacket, Packable};
use render::{Renderer};
use ship::Ship;
use sim_element::SimElement;

// Use+reexport all of the modules
pub use self::engine::EngineModule;
pub use self::proj_weapon::ProjectileWeaponModule;

pub mod engine;
pub mod proj_weapon;

///////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleRef = Rc<RefCell<Module>>;

pub enum Module {
    Engine(EngineModule),
    ProjectileWeapon(ProjectileWeaponModule),
}

impl Module {
    pub fn get_base<'a>(&'a mut self) -> &'a mut ModuleBase {
        match (*self) {
            Engine(ref mut m) => &mut m.base,
            ProjectileWeapon(ref mut m) => &mut m.base,
        }
    }
    
    fn get_type_id(&self) -> u16 {
        match *self {
            Engine(_) => 0,
            ProjectileWeapon(_) => 1,
        }
    }
}

impl SimElement for Module {
    fn server_preprocess(&mut self, ships: &HashMap<ClientId, Ship>) {
        match *self {
            Engine(ref mut m) => m.server_preprocess(ships),
            ProjectileWeapon(ref mut m) => m.server_preprocess(ships),
        }
    }
    
    fn before_simulation(&mut self, ships: &HashMap<ClientId, Ship>) {
        match *self {
            Engine(ref mut m) => m.before_simulation(ships),
            ProjectileWeapon(ref mut m) => m.before_simulation(ships),
        }
    }
    
    fn on_simulation_time(&mut self, ships: &HashMap<ClientId, Ship>, time: u32) {
        match *self {
            Engine(ref mut m) => m.on_simulation_time(ships, time),
            ProjectileWeapon(ref mut m) => m.on_simulation_time(ships, time),
        }
    }
    
    fn after_simulation(&mut self, ships: &HashMap<ClientId, Ship>) {
        match *self {
            Engine(ref mut m) => m.after_simulation(ships),
            ProjectileWeapon(ref mut m) => m.after_simulation(ships),
        }
    }
    
    fn get_critical_times(&self) -> Vec<u32> {
        match *self {
            Engine(ref m) => m.get_critical_times(),
            ProjectileWeapon(ref m) => m.get_critical_times(),
        }
    }
    
    fn draw(&self, renderer: &mut Renderer, simulating: bool, time: f32) {
        match *self {
            Engine(ref m) => m.draw(renderer, simulating, time),
            ProjectileWeapon(ref m) => m.draw(renderer, simulating, time),
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

pub fn read_module_from_packet(packet: &mut InPacket) -> IoResult<Module> {
    let module_type = try!(packet.read_u16());
    match module_type {
        0 => Ok(Engine(try!(packet.read::<EngineModule>()))),
        1 => Ok(ProjectileWeapon(try!(packet.read::<ProjectileWeaponModule>()))),
        _ => {Err(IoError{kind: InvalidInput, desc: "Failed to read module with invalid module type ID", detail: None})}
    }
}

pub fn write_module_to_packet(module: &Module, packet: &mut OutPacket) -> IoResult<()> {
    packet.write_u16(module.get_type_id()).unwrap();
    match *module {
        Engine(ref module) => try!(module.write_to_packet(packet)),
        ProjectileWeapon(ref module) => try!(module.write_to_packet(packet)),
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleBase {
    // Module position/texture stuff
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,

    // Module stats
    pub power: u32,
    pub max_power: u32,
    pub damage: u32,
    pub hull: u32,
}

impl ModuleBase {
    pub fn new() -> ModuleBase {
        ModuleBase{x: 0, y: 0, width: 1, height: 1, power: 0, max_power: 1, damage: 0, hull: 0}
    }
}

impl Packable for ModuleBase {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<ModuleBase> {
        Ok(ModuleBase {
            x: try!(packet.read_u8()),
            y: try!(packet.read_u8()),
            width: try!(packet.read_u8()),
            height: try!(packet.read_u8()),
            
            power: try!(packet.read_u32()),
            max_power: try!(packet.read_u32()),
            damage: try!(packet.read_u32()),
            hull: try!(packet.read_u32())
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write_u8(self.x));
        try!(packet.write_u8(self.y));
        try!(packet.write_u8(self.width));
        try!(packet.write_u8(self.height));
        
        try!(packet.write_u32(self.power));
        try!(packet.write_u32(self.max_power));
        try!(packet.write_u32(self.damage));
        try!(packet.write_u32(self.hull));
        Ok(())
    }
}