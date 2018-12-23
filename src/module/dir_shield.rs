use std::collections::HashMap;

use module::{IModule, Model, ModelIndex, Module, ModuleClass, ModuleContext, ModuleShape, TargetManifest};
use ship::ShipState;

#[cfg(feature = "client")]
use sim_visuals::SpriteVisual;
#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(Serialize, Deserialize, Clone)]
pub struct DirShieldModule {
    base_sprite: String,
}

impl DirShieldModule {
    pub fn new(model: ModelIndex) -> Module {
        Module::new(
            model, ModuleShape::new(vec![vec![b'#']]),
            2, 2, 3, DirShieldModule { base_sprite: "shiel".to_owned() })
    }

    pub fn from_properties(model: &Model, prop: &HashMap<String, String>) -> Module {    
        Module::from_model(model,
            DirShieldModule {
                base_sprite: prop[&"base".to_string()].clone(),
            },
        )
    }
}

impl IModule for DirShieldModule {
    fn get_class(&self) -> ModuleClass { ModuleClass::DirShield }

    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let mut shield_sprite = SpriteSheet::new(asset_store.get_sprite_info_str(&self.base_sprite));
        
        /*if context.is_active {
            shield_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 9, 0.05));
        } else {*/
            shield_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        //}
    
        effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), 0.0, shield_sprite));
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        self.add_plan_effects(context, asset_store, effects);
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
        if ship_state.shields < ship_state.max_shields {
            ship_state.shields += 1; // charge shield
        }
    }
    
    fn on_activated(&mut self, ship_state: &mut ShipState) {
        ship_state.add_shields(2);
    }
    
    fn on_deactivated(&mut self, ship_state: &mut ShipState) {
        ship_state.remove_shields(2);
    }
}
