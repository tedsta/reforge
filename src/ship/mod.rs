use std::rc::Rc;
use std::cell::RefCell;
use std::default::Default;
use std::io::IoResult;

use module::{ModuleRef, Module, read_module_from_packet, write_module_to_packet};
use net::{ClientId, InPacket, OutPacket, Packable};
use render::{RenderTarget};
use self::ship_gen::generate_ship;

// Use the ship_gen module privately here
mod ship_gen;

// Holds everything about the ship's damage, capabilities, etc.
pub struct ShipState {
    pub engines: uint,
    pub shields: uint,
    pub max_shields: uint,
}

impl ShipState {
    pub fn new() -> ShipState {
        ShipState{engines: 0, shields: 0, max_shields: 0}
    }
}

impl Packable for ShipState {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<ShipState> {
        Ok(ShipState {
            engines: try!(packet.read_u32()) as uint,
            shields: try!(packet.read_u32()) as uint,
            max_shields: try!(packet.read_u32()) as uint,
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write_u32(self.engines as u32));
        try!(packet.write_u32(self.shields as u32));
        try!(packet.write_u32(self.max_shields as u32));
        Ok(())
    }
}

#[deriving(Clone)]
pub struct ShipRef {
    ship: Rc<RefCell<Ship>>,
}

// Type for the ID of a ship
pub type ShipId = u32;

pub struct Ship {
    pub id: ClientId,
    pub state: ShipState,
    pub modules: Vec<ModuleRef>,
    pub render_target: RenderTarget,
}

impl ShipRef {
    pub fn new(id: ClientId) -> ShipRef {
        ShipRef{ship: Rc::new(RefCell::new(Ship{id: id, state: ShipState::new(), modules: vec!(), render_target: Default::default()}))}
    }
    
    pub fn generate(id: ClientId) -> ShipRef {
        generate_ship(id)
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module(&self, mut module: Module) -> bool {
        module.get_base().ship = Some(self.clone());
        self.borrow_mut().modules.push(Rc::new(RefCell::new(module)));
        true
    }
}

impl Packable for ShipRef {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<ShipRef> {
        let id = try!(packet.read_u32());
        let state: ShipState = try!(packet.read());
        
        // Deserialize modules
        let num_modules = try!(packet.read_u8()) as uint;
        
        let mut ship = ShipRef{ship: Rc::new(RefCell::new(Ship {
            id: id,
            state: state,
            modules: Vec::with_capacity(num_modules),
            render_target: Default::default(),
        }))};
        
        for _ in range(0, num_modules) {
            ship.add_module(try!(read_module_from_packet(packet)));
        }
        
        Ok(ship)
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        let ship = self.borrow();
    
        try!(packet.write_u32(ship.id));
        try!(ship.state.write_to_packet(packet));
        try!(packet.write_u8(ship.modules.len() as u8));
        for module in ship.modules.iter() {
            try!(write_module_to_packet(module.borrow().deref(), packet));
        }
        Ok(())
    }
}

impl Deref<Rc<RefCell<Ship>>> for ShipRef {
    fn deref<'a>(&'a self) -> &'a Rc<RefCell<Ship>> {
        &self.ship
    }
}