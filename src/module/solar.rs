#[cfg(client)]
use graphics::Context;
#[cfg(client)]
use opengl_graphics::Gl;

use battle_state::BattleContext;
use assets::SOLAR_TEXTURE;
use module::{IModule, Module, ModuleBase, ModuleRef, ModuleType, ModuleTypeStore, Propulsion, Solar};
use net::{InPacket, OutPacket};
use ship::{ShipRef, ShipState};
use sim::SimEventAdder;
use vec::{Vec2, Vec2f};

#[cfg(client)]
use sim::{SimVisuals, SimVisual};
#[cfg(client)]
use sprite_sheet::{SpriteSheet, Loop};
#[cfg(client)]
use asset_store::AssetStore;

#[deriving(Encodable, Decodable)]
pub struct SolarModule {
    pub base: ModuleBase,
}

impl SolarModule {
    pub fn new(mod_store: &ModuleTypeStore, mod_type: ModuleType) -> Module {
        Solar(SolarModule {
            base: ModuleBase::new(mod_store, mod_type, 3),
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
        solar_sprite.add_animation(Loop(0.0, 5.0, 1, 4, 0.1));
    
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