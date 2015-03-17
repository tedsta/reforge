#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

use battle_state::BattleContext;
use assets::BEAM_WEAPON_TEXTURE;
use module;
use module::{IModule, Module, ModuleBase, ModuleRef};
use net::{InPacket, OutPacket};
use ship::{ShipRef, ShipState};
use sim::SimEventAdder;
use sim_events::DamageEvent;
use sim_visuals::{BeamExitVisual, BeamVisual, SpriteVisual};
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
            base: ModuleBase::new(1, 1, 3, 2, 3),
            module: BeamWeaponModule,
        }
    }
}

impl IModule for BeamWeaponModule {
    fn server_preprocess(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
    }
    
    fn before_simulation(&mut self, base: &mut ModuleBase, ship: &ShipRef, events: &mut SimEventAdder) {
        if base.powered {
            if let Some(ref target) = base.target {
                let ref target_ship = target.ship;
            
                if let module::TargetData::Beam(beam_start, beam_end) = target.data {
                    target_ship.borrow().beam_hits(beam_start, beam_end, |module, _, _, hit| {
                        if let Some(hit_dist) = hit {
                            let hit_tick = 20 + (((3.0 - 1.0)*hit_dist*20.0) as u32);
                        
                            events.add(hit_tick, box DamageEvent::new(target_ship.clone(), module.clone(), 1));
                        }
                    });
                }
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, base: &ModuleBase, asset_store: &AssetStore, effects: &mut SimEffects, ship: &ShipRef) {
        let mut sprite = SpriteSheet::new(asset_store.get_sprite_info(BEAM_WEAPON_TEXTURE));

        if base.is_active() {
            sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 1, 23, 0.2));
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
        
        let ship_id = ship.borrow().id;
        
        if base.powered {
            if let Some(ref target) = base.target {
                let target_ship_id = target.ship.borrow().id;
            
                if let module::TargetData::Beam(beam_start, beam_end) = target.data {
                    let start_time = 1.0;
                    let end_time = 3.0;
                    
                    // Add the simulation visual for beam leaving ship screen
                    effects.add_visual(ship_id, 2, box BeamExitVisual {
                        start_time: start_time,
                        end_time: end_time,
                        
                        beam_start: base.get_render_center() + Vec2 { x: 20.0, y: 0.0 },
                        beam_end: base.get_render_center() + Vec2 { x: 20.0, y: 0.0 } + Vec2 { x: 1500.0, y: 0.0 },
                    });
                    
                    // Add the simulation visual for beam entering target screen
                    effects.add_visual(target_ship_id, 2, box BeamVisual {
                        start_time: start_time,
                        end_time: end_time,
                        
                        beam_start: beam_start,
                        beam_end: beam_end,
                    });
                }
            }
        }
    }
    
    fn after_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
    }
    
    fn on_activated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
    }
    
    fn on_deactivated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
    }
    
    fn get_target_mode(&self, base: &ModuleBase) -> Option<module::TargetMode> {
        Some(module::TargetMode::Beam(3))
    }
}