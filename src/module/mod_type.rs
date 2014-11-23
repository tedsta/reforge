use assets::{TextureId, ENGINE_TEXTURE, LASER_TEXTURE};
use module::{ModuleCategory, Weapon, Propulsion, Defense, Power};

pub type ModuleType = u16;

pub struct ModuleTypeInfo {
    pub category: ModuleCategory,
    pub texture: TextureId,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleTypeStore {
    module_info: Vec<ModuleTypeInfo>,
}

impl ModuleTypeStore {
    pub fn new() -> ModuleTypeStore {
        let module_info = vec![
            ModuleTypeInfo{category: Propulsion, texture: ENGINE_TEXTURE},
            ModuleTypeInfo{category: Weapon, texture: LASER_TEXTURE},
            ModuleTypeInfo{category: Defense, texture: LASER_TEXTURE},
            ModuleTypeInfo{category: Power, texture: LASER_TEXTURE},
        ];
    
        ModuleTypeStore {
            module_info: module_info,
        }
    }
    
    pub fn get_module_type<'a>(&'a self, mod_type: ModuleType) -> &'a ModuleTypeInfo {
        &self.module_info[mod_type as uint]
    }
}