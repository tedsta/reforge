#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::GlGraphics;

use battle_context::BattleContext;
use module;
use module::{IModule, Module, ModuleClass, ModuleContext, ModuleShape, TargetManifest};
use net::{InPacket, OutPacket};
use ship::{Ship, ShipState};
use sim::SimEvents;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim_visuals::SpriteVisual;
#[cfg(feature = "client")]
use sim::{SimEffects, SimVisual};
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct EngineModule;

impl EngineModule {
    pub fn new() -> Module {
        Module::new(ModuleShape::new(vec![vec![b'#', b'#'],
                                          vec![b'.', b'.']]), 2, 2, 3, EngineModule)
    }
}

impl IModule for EngineModule {
    fn get_class(&self) -> ModuleClass { ModuleClass::Engine }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let mut engine_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("engine1"));
        engine_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
    
        effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), 0.0, engine_sprite));
        
        // Propulsion sprite
        if context.is_active {
            let mut prop_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("propulsion1"));
            prop_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 7, 0.05));
        
            effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position() + Vec2{x: -48.0, y: 2.0}, 0.0, prop_sprite));
        }
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        self.add_plan_effects(context, asset_store, effects);
    }
    
    fn on_activated(&mut self, ship_state: &mut ShipState) {
        ship_state.thrust += 1;
    }
    
    fn on_deactivated(&mut self, ship_state: &mut ShipState) {
        ship_state.thrust -= 1;
    }
}
