#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

use battle_state::BattleContext;
use assets::WEAPON_TEXTURE;
use module;
use module::{IModule, Module, ModuleBase, ModuleRef};
use net::{InPacket, OutPacket};
use ship::{ShipRef, ShipState};
use sim::SimEventAdder;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim::{SimEffects, SimVisual};
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct BeamWeaponModule;

impl BeamWeaponModule {
    pub fn new() -> Module<BeamWeaponModule> {
        Module {
            base: ModuleBase::new(1, 2, 0, 2, 4),
            module: BeamWeaponModule,
        }
    }
}

impl IModule for BeamWeaponModule {
    fn server_preprocess(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
    }
    
    fn before_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, events: &mut SimEventAdder) {
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, base: &ModuleBase, asset_store: &AssetStore, effects: &mut SimEffects, ship: &ShipRef) {
        let mut sprite = SpriteSheet::new(asset_store.get_sprite_info(WEAPON_TEXTURE));

        if base.is_active() {
            sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 7, 0.2));
        } else {
            sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        effects.add_visual(ship.borrow().id, 0, box SpriteVisual {
            position: base.get_render_position().clone(),
            sprite_sheet: sprite,
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

// Sprite sheet sim visual
#[cfg(feature = "client")]
pub struct SpriteVisual {
    position: Vec2f,
    sprite_sheet: SpriteSheet,
}

#[cfg(feature = "client")]
impl SimVisual for SpriteVisual {
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64) {
        self.sprite_sheet.draw(context, gl, self.position.x, self.position.y, 0.0, time);
    }
}