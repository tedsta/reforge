#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

use battle_state::BattleContext;
use assets::SHIELD_TEXTURE;
use module;
use module::{IModule, Module, ModuleBase, ModuleRef};
use net::{InPacket, OutPacket};
use ship::{ShipRef, ShipState};
use space_gui;
use sim::SimEventAdder;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim::{SimVisuals, SimVisual};
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct ShieldModule;

impl ShieldModule {
    pub fn new() -> Module<ShieldModule>{
        Module {
            base: ModuleBase::new(1, 1, 2, 2, 3),
            module: ShieldModule,
        }
    }
}

impl IModule for ShieldModule {
    fn server_preprocess(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
    }
    
    fn before_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, events: &mut SimEventAdder) {
    }
    
    #[cfg(feature = "client")]
    fn add_plan_visuals(&self, base: &ModuleBase, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        let mut shield_sprite = SpriteSheet::new(asset_store.get_sprite_info(SHIELD_TEXTURE));
        
        if base.is_active() {
            shield_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 9, 0.05));
        } else {
            shield_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        visuals.add(ship.borrow().id, 0, box SpriteVisual {
            position: base.get_render_position().clone(),
            sprite_sheet: shield_sprite,
        });
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_visuals(&self, base: &ModuleBase, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        self.add_plan_visuals(base, asset_store, visuals, ship);
    }
    
    fn after_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
        if base.powered && ship_state.shields < ship_state.max_shields {
            ship_state.shields += 1; // charge shield
        }
    }

    fn write_plans(&self, base: &ModuleBase, packet: &mut OutPacket) {
    }
    
    fn read_plans(&mut self, base: &mut ModuleBase, context: &BattleContext, packet: &mut InPacket) {
    }
    
    fn write_results(&self, base: &ModuleBase, packet: &mut OutPacket) {
    }
    
    fn read_results(&mut self, base: &mut ModuleBase, packet: &mut InPacket) {
    }
    
    fn on_activated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
        ship_state.add_shields(2);
    }
    
    fn on_deactivated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
        ship_state.remove_shields(2);
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