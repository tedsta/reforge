#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

use battle_context::BattleContext;
use module;
use module::{IModule, Module, ModuleClass, TargetManifest};
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
pub struct SolarModule;

impl SolarModule {
    pub fn new() -> Module {
        Module::new(1, 1, 0, 2, 3, SolarModule)
    }
}

impl IModule for SolarModule {
    fn get_class(&self) -> ModuleClass { ModuleClass::Solar }

    #[cfg(feature = "client")]
    fn add_plan_effects(&self, base: &Module, asset_store: &AssetStore, effects: &mut SimEffects, ship: &Ship) {
        let mut solar_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("modules/solar_panel_sprite.png"));
        
        if base.is_active() {
            solar_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 1, 4, 0.1));
        } else {
            solar_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        effects.add_visual(ship.id, 0, box SpriteVisual {
            position: base.get_render_position().clone(),
            sprite_sheet: solar_sprite,
        });
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, base: &Module, asset_store: &AssetStore, effects: &mut SimEffects, ship: &Ship, target: Option<TargetManifest>) {
        self.add_plan_effects(base, asset_store, effects, ship);
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
    }
    
    fn on_activated(&mut self, ship_state: &mut ShipState) {
        ship_state.add_power(5);
    }
    
    fn on_deactivated(&mut self, ship_state: &mut ShipState) {
        ship_state.remove_power(5);
    }
}
