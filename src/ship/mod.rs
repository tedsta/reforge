use std::rc::Rc;
use std::cell::RefCell;

use serialize::{Encodable, Encoder, Decodable, Decoder};

use module::{IModule, ModuleRef, Module};
use net::{ClientId, InPacket, OutPacket};
use self::ship_gen::generate_ship;
use sim::SimEvents;

#[cfg(client)]
use sfml_renderer::SfmlRenderer;

// Use the ship_gen module privately here
mod ship_gen;

// Holds everything about the ship's damage, capabilities, etc.
#[deriving(Encodable, Decodable)]
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

pub type ShipRef = Rc<RefCell<Ship>>;

// Type for the ID of a ship
pub type ShipId = u64;

pub struct Ship {
    pub id: ShipId,
    pub client_id: Option<ClientId>,
    pub state: ShipState,
    pub modules: Vec<ModuleRef>,
}

impl Ship {
    pub fn new(id: ShipId) -> Ship {
        Ship {
            id: id,
            client_id: None,
            state: ShipState::new(),
            modules: vec!(),
        }
    }
    
    pub fn generate(id: ShipId) -> Ship {
        generate_ship(id)
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module(&mut self, module: Module) -> bool {
        self.modules.push(Rc::new(RefCell::new(module)));
        true
    }
    
    pub fn server_preprocess(&mut self) {
        for module in self.modules.iter() {
            module.borrow_mut().server_preprocess(&mut self.state);
        }
    }
    
    pub fn before_simulation(&mut self, events: &mut SimEvents) {
        for module in self.modules.iter() {
            module.borrow_mut().before_simulation(&mut self.state, &mut events.create_adder(module.clone()));
        }
    }
    
    pub fn after_simulation(&mut self) {
        for module in self.modules.iter() {
            module.borrow_mut().after_simulation(&mut self.state);
        }
    }
    
    pub fn write_plans(&self, packet: &mut OutPacket) {
        for module in self.modules.iter() {
            module.borrow().write_plans(packet);
        }
    }
    
    pub fn read_plans(&self, packet: &mut InPacket) {
        for module in self.modules.iter() {
            module.borrow_mut().read_plans(packet);
        }
    }
    
    pub fn write_results(&self, packet: &mut OutPacket) {
        for module in self.modules.iter() {
            module.borrow().write_results(packet);
        }
    }
    
    pub fn read_results(&self, packet: &mut InPacket) {
        for module in self.modules.iter() {
            module.borrow_mut().read_results(packet);
        }
    }
    
    #[cfg(client)]
    pub fn draw(&self, renderer: &SfmlRenderer) {
        for module in self.modules.iter() {
            module.borrow().get_base().draw(renderer, self);
        }
    }
}

impl <S: Encoder<E>, E> Encodable<S, E> for Ship {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
        encoder.emit_struct("Ship", 0, |encoder| {
            try!(encoder.emit_struct_field("id", 0, |encoder|self.id.encode(encoder)));
            try!(encoder.emit_struct_field("client_id", 1, |encoder|self.client_id.encode(encoder)));
            try!(encoder.emit_struct_field("state", 2, |encoder| self.state.encode(encoder)));
            try!(encoder.emit_struct_field("modules", 3, |encoder| self.modules.encode(encoder)));
            Ok(())
        })
    }
}

impl<S: Decoder<E>, E> Decodable<S, E> for Ship {
  fn decode(decoder: &mut S) -> Result<Ship, E> {
    decoder.read_struct("root", 0, |decoder| {
        let ship = Ship{
            id: try!(decoder.read_struct_field("id", 0, |decoder| Decodable::decode(decoder))),
            client_id: try!(decoder.read_struct_field("client_id", 0, |decoder| Decodable::decode(decoder))),
            state: try!(decoder.read_struct_field("state", 0, |decoder| Decodable::decode(decoder))),
            modules: try!(decoder.read_struct_field("modules", 0, |decoder| Decodable::decode(decoder))),
        };
        Ok(ship)
    })
  }
}