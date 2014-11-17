use std::rc::Rc;

#[cfg(client)]
use opengl_graphics::Texture;

pub type TextureId = u16;

pub static ENGINE_TEXTURE: u16 = 0;
pub static WEAPON_TEXTURE: u16 = 1;
pub static SHIELD_TEXTURE: u16 = 2;
pub static LASER_TEXTURE: u16 = 3;
pub static EXPLOSION_TEXTURE: u16 = 4;
pub static GUI_TEXTURE: u16 = 5;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(client)]
pub struct SpriteInfo {
    pub texture: Rc<Texture>,
    pub columns: u8, 
    pub rows: u8,
}