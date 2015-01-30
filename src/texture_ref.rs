use std::rc::Rc;

use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use opengl_graphics::Texture;

pub struct TextureRef(Rc<Texture>);

impl Decodable for TextureRef {
    fn decode<D: Decoder>(d: &mut D) -> Result<TextureRef, D::Error> { Ok() }
}

impl Encodable for TextureRef {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> { Ok(()) }
}