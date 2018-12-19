use battle_context::BattleContext;
use module;
use module::{IModule, ModelIndex, Module, ModuleClass, ModuleContext, ModuleShape, TargetManifest};
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

#[derive(Serialize, Deserialize, Clone)]
pub struct CabinModule;

impl CabinModule {
    pub fn new(model: ModelIndex) -> Module {
        Module::new(model, ModuleShape::new(vec![vec![b'#', b'.', b'.'],
                                                 vec![b'#', b'#', b'.'],
                                                 vec![b'#', b'.', b'.']]), 0, 2, 4, CabinModule)
    }
}

impl IModule for CabinModule {
    fn get_class(&self) -> ModuleClass { ModuleClass::Cabin }

    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let mut command_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("cabin"));

        if context.is_active {
            command_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 7, 0.2));
        } else {
            command_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), 0.0, command_sprite));
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        self.add_plan_effects(context, asset_store, effects);
    }
}
