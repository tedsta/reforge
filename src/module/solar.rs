#[cfg(client)]
use graphics::Context;
#[cfg(client)]
use opengl_graphics::Gl;

use battle_state::BattleContext;
use assets::SOLAR_TEXTURE;
use module::{IModule, Module, ModuleBase, ModuleRef};
use net::{InPacket, OutPacket};
use ship::{ShipRef, ShipState};
use sim::SimEventAdder;
use vec::{Vec2, Vec2f};

#[cfg(client)]
use sim::{SimVisuals, SimVisual};
#[cfg(client)]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(client)]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable)]
pub struct SolarModule {
    pub base: ModuleBase,
}

impl SolarModule {
    pub fn new() -> Module {
        Module::Solar(SolarModule {
            base: ModuleBase::new(1, 1, 0, 2, 3),
        })
    }
}

impl IModule for SolarModule {
    fn server_preprocess(&mut self, ship_state: &mut ShipState) {
    }
    
    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder) {
    }
    
    #[cfg(client)]
    fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        let mut solar_sprite = SpriteSheet::new(asset_store.get_sprite_info(SOLAR_TEXTURE));
        
        if self.base.is_active() {
            solar_sprite.add_animation(SpriteAnimation::Loop(0.0, 5.0, 1, 4, 0.1));
        } else {
            solar_sprite.add_animation(SpriteAnimation::Stay(0.0, 5.0, 0));
        }
    
        visuals.add(ship.borrow().id, 0, box SpriteVisual {
            position: self.base.get_render_position().clone(),
            sprite_sheet: solar_sprite,
        });
    }
    
    #[cfg(client)]
    fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        self.add_plan_visuals(asset_store, visuals, ship);
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
    }
    
    fn read_plans(&mut self, context: &BattleContext, packet: &mut InPacket) {
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
    }
    
    fn on_activated(&mut self, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
        ship_state.add_power(5);
    }
    
    fn on_deactivated(&mut self, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
        ship_state.remove_power(5, modules);
    }
    
    fn on_icon_clicked(&mut self) -> bool {
        false
    }
    
    fn on_module_clicked(&mut self, ship: &ShipRef, module: &ModuleRef) -> bool {
        false
    }
}

// Sprite sheet sim visual
#[cfg(client)]
pub struct SpriteVisual {
    position: Vec2f,
    sprite_sheet: SpriteSheet,
}

#[cfg(client)]
impl SimVisual for SpriteVisual {
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64) {
        self.sprite_sheet.draw(context, gl, self.position.x, self.position.y, 0.0, time);
    }
}
