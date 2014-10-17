use std::rc::Rc;
use std::cell::RefCell;
use std::default::Default;
use std::io::IoResult;

use serialize::{Encodable, Encoder, Decodable, Decoder};

use module::{ModuleRef, Module, ModuleCategory};
use net::{ClientId, InPacket, OutPacket};
use render::{RenderTarget};
use self::ship_gen::generate_ship;

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

// Type for the ID of a ship
pub type ShipId = u64;

// Index to access ship
#[deriving(Encodable, Decodable, Show)]
pub struct ShipIndex {
    pub id: ShipId,
    pub index: Option<u16>,
}

pub struct Ship {
    pub index: ShipIndex,
    pub client_id: Option<ClientId>,
    pub state: ShipState,
    pub modules: Vec<ModuleRef>,
    pub render_target: RenderTarget,
}

impl Ship {
    pub fn new(id: ShipId) -> Ship {
        Ship {
            index: ShipIndex{id: id, index: None},
            client_id: None,
            state: ShipState::new(),
            modules: vec!(),
            render_target: Default::default()
        }
    }
    
    pub fn generate(id: ShipId) -> Ship {
        generate_ship(id)
    }
    
    // Returns true if adding the module was successful, false if it failed.
    pub fn add_module(&mut self, mut module: Module) -> bool {
        self.modules.push(Rc::new(RefCell::new(module)));
        true
    }
}

impl <S: Encoder<E>, E> Encodable<S, E> for Ship {
  fn encode(&self, encoder: &mut S) -> Result<(), E> {
        encoder.emit_struct("Ship", 0, |encoder| {
            try!(encoder.emit_struct_field("index", 0, |encoder|self.index.encode(encoder)));
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
            index: try!(decoder.read_struct_field("index", 0, |decoder| Decodable::decode(decoder))),
            client_id: try!(decoder.read_struct_field("client_id", 0, |decoder| Decodable::decode(decoder))),
            state: try!(decoder.read_struct_field("state", 0, |decoder| Decodable::decode(decoder))),
            modules: try!(decoder.read_struct_field("modules", 0, |decoder| Decodable::decode(decoder))),
            render_target: Default::default(),
        };
        Ok(ship)
    })
  }
}