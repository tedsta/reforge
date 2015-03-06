#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

use battle_state::BattleContext;
use assets::COMMAND_TEXTURE;
use module;
use module::{IModule, Module, ModuleBase, ModuleRef};
use net::{InPacket, OutPacket};
use ship::{ShipRef, ShipState};
use sim::SimEventAdder;
use sim_visuals::SpriteVisual;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim::{SimEffects, SimVisual};
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct CommandModule;

impl CommandModule {
    pub fn new() -> Module<CommandModule> {
        Module {
            base: ModuleBase::new(1, 2, 0, 2, 4),
            module: CommandModule,
        }
    }
}

impl IModule for CommandModule {
    fn server_preprocess(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
    }
    
    fn before_simulation(&mut self, base: &mut ModuleBase, ship: &ShipRef, events: &mut SimEventAdder) {
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, base: &ModuleBase, asset_store: &AssetStore, effects: &mut SimEffects, ship: &ShipRef) {
        let mut command_sprite = SpriteSheet::new(asset_store.get_sprite_info(COMMAND_TEXTURE));

        if base.is_active() {
            command_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 7, 0.2));
        } else {
            command_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        effects.add_visual(ship.borrow().id, 0, box SpriteVisual {
            position: base.get_render_position().clone(),
            sprite_sheet: command_sprite,
        });
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, base: &ModuleBase, asset_store: &AssetStore, effects: &mut SimEffects, ship: &ShipRef) {
        self.add_plan_effects(base, asset_store, effects, ship);
    }
    
    fn after_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
    }
    
    fn on_activated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
    }
    
    fn on_deactivated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
    }
    
    fn get_target_mode(&self, base: &ModuleBase) -> Option<module::TargetMode> {
        None
    }
}