use module::{IModule, Module, ModuleClass, ModuleContext, ModuleShape, TargetManifest};
use ship::ShipState;

#[cfg(feature = "client")]
use sim_visuals::SpriteVisual;
#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct ShieldModule;

impl ShieldModule {
    pub fn new() -> Module {
        Module::new(ModuleShape::new(vec![vec![b'#']]), 2, 2, 3, ShieldModule)
    }
}

impl IModule for ShieldModule {
    fn get_class(&self) -> ModuleClass { ModuleClass::Shield }

    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let mut shield_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("modules/shield_sprite.png"));
        
        if context.is_active {
            shield_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 9, 0.05));
        } else {
            shield_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
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
