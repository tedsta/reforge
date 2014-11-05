pub type TextureId = u16;

pub static ENGINE_TEXTURE: u16 = 0;
pub static WEAPON_TEXTURE: u16 = 1;
pub static LASER_TEXTURE: u16 = 2;
pub static EXPLOSION_TEXTURE: u16 = 3;
pub static GUI_TEXTURE: u16 = 4;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SpriteInfo {
    pub texture: TextureId,
    pub texture_width: u16,
    pub texture_height: u16,
    pub columns: u8, 
    pub rows: u8,
}