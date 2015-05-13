use module::{IModule, Module, ModuleClass, ModuleContext, TargetManifest, TargetManifestData, TargetMode};
use ship::ShipState;
use sim::SimEvents;
use sim_events::RepairEvent;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim_visuals::SpriteVisual;
#[cfg(feature = "client")]
use sim::SimEffects;
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct RepairModule;

impl RepairModule {
    pub fn new() -> Module {
        Module::new(1, 1, 2, 2, 3, RepairModule)
    }
}

impl IModule for RepairModule {
    fn get_class(&self) -> ModuleClass { ModuleClass::Repair }
    
    fn get_target_mode(&self) -> Option<TargetMode> {
        Some(TargetMode::OwnModule)
    }
    
    fn before_simulation(&mut self, context: &ModuleContext, events: &mut SimEvents) {
        if let Some(ref target) = context.target {
            if let TargetManifestData::OwnModule(module) = target.data {
                events.add(40,
                           target.ship.index,
                           Box::new(RepairEvent::new(module.index, 1)));
                
                events.add(80,
                           target.ship.index,
                           Box::new(RepairEvent::new(module.index, 1)));
            }
        }
    }

    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let mut sprite = SpriteSheet::new(asset_store.get_sprite_info_str("modules/repair_sprite.png"));
        
        if context.is_active {
            sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 1, 18, 0.055));
        } else {
            sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), sprite));
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        self.add_plan_effects(context, asset_store, effects);
    }
    
    fn on_activated(&mut self, ship_state: &mut ShipState) {
    }
    
    fn on_deactivated(&mut self, ship_state: &mut ShipState) {
    }
}
