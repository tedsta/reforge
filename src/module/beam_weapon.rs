#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

use battle_context::BattleContext;
use module;
use module::{IModule, Module, ModuleBase, ModuleRef, TargetManifest, TargetManifestData};
use net::{InPacket, OutPacket};
use ship::{ShipRef, ShipState};
use sim::SimEvents;
use sim_events::DamageEvent;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim_visuals::{BeamExitVisual, BeamVisual, SpriteVisual};
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
    fn server_preprocess(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, target: Option<TargetManifest>) {
    }
    
    fn before_simulation(&mut self, base: &mut ModuleBase, events: &mut SimEvents, target: Option<TargetManifest>) {
        if base.powered {
            if let Some(ref target) = target {
                if let module::TargetManifestData::Beam(beam_start, beam_end) = target.data {
                    target.ship.borrow().beam_hits(beam_start, beam_end, |module, _, _, hit| {
                        if let Some(hit_dist) = hit {
                            let hit_tick = 20 + (((3.0 - 1.0)*hit_dist*20.0) as u32);
                        
                            events.add(
                                hit_tick,
                                target.ship.borrow().index,
                                box DamageEvent::new(module.borrow().get_base().index, 1),
                            );
                        }
                    });
                }
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, base: &ModuleBase, asset_store: &AssetStore, effects: &mut SimEffects, ship: &ShipRef) {
        let mut sprite = SpriteSheet::new(asset_store.get_sprite_info_str("modules/small_beam_sprite.png"));

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
    fn add_simulation_effects(&self, base: &ModuleBase, asset_store: &AssetStore, effects: &mut SimEffects, ship: &ShipRef, target: Option<TargetManifest>) {
        self.add_plan_effects(base, asset_store, effects, ship);
        
        let ship_id = ship.borrow().id;
        
        if base.powered {
            if let Some(ref target) = target {
                let target_ship_id = target.ship.borrow().id;
            
                if let module::TargetManifestData::Beam(beam_start, beam_end) = target.data {
                    let start_time = 1.0;
                    let end_time = 3.0;
                    
                    // Add the simulation visual for beam leaving ship screen
                    effects.add_visual(ship_id, 2, box BeamExitVisual {
                        start_time: start_time,
                        end_time: end_time,
                        
                        beam_start: base.get_render_center() + Vec2 { x: 12.0, y: 0.0 },
                        
                        texture: asset_store.get_texture_str("effects/small_beam_part.png").clone(),
                    });
                    
                    // Add the simulation visual for beam entering target screen
                    let mut beam_end_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("effects/small_beam_end.png"));
                    beam_end_sprite.add_animation(SpriteAnimation::Loop(0.0, 2.0, 0, 3, 0.1)); // 2 second beam duration
                    
                    let beam_visual =
                        BeamVisual::new(
                            start_time, end_time,
                            beam_start, beam_end,
                            asset_store.get_texture_str("effects/small_beam_part.png").clone(),
                            beam_end_sprite
                        );
                    
                    effects.add_visual(target_ship_id, 2, box beam_visual);
                    
                    effects.add_sound(start_time, 1, asset_store.get_sound(&"effects/beam1.ogg".to_string()).clone());
                }
            }
        }
    }
    
    fn after_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
    }
    
    fn get_target_mode(&self, base: &ModuleBase) -> Option<module::TargetMode> {
        Some(module::TargetMode::Beam(3))
    }
}
