use std::rc::Rc;

#[cfg(feature = "client")]
use opengl_graphics::Texture;

pub type TextureId = u16;

pub static ENGINE_TEXTURE: u16 = 0;
pub static WEAPON_TEXTURE: u16 = 1;
pub static SHIELD_TEXTURE: u16 = 2;
pub static SOLAR_TEXTURE: u16 = 3;
pub static REPAIR_TEXTURE: u16 = 4;
pub static COMMAND_TEXTURE: u16 = 5;
pub static LASER_TEXTURE: u16 = 6;
pub static EXPLOSION_TEXTURE: u16 = 7;
pub static PROPULSION_TEXTURE: u16 = 8;
pub static FIRE_TEXTURE: u16 = 9;
pub static SMOKE_TEXTURE: u16 = 10;
pub static GUI_TEXTURE: u16 = 11;
pub static BEAM_WEAPON_TEXTURE: u16 = 12;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "client")]
pub struct SpriteInfo {
    pub texture: Rc<Texture>,
    pub columns: u8, 
    pub rows: u8,
}