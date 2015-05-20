#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::GlGraphics;

use battle_context::BattleContext;
use module;
use module::{IModule, Module, ModuleClass, ModuleContext, TargetManifest, TargetManifestData};
use net::{InPacket, OutPacket};
use ship::{Ship, ShipState};
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
    pub fn new() -> Module {
        Module::new(1, 1, 3, 2, 3, BeamWeaponModule)
    }
}

impl IModule for BeamWeaponModule {
    fn get_class(&self) -> ModuleClass { ModuleClass::BeamWeapon }
    
    fn get_target_mode(&self) -> Option<module::TargetMode> {
        Some(module::TargetMode::Beam(3))
    }

    fn before_simulation(&mut self, context: &ModuleContext, events: &mut SimEvents) {
        if let Some(ref target) = context.target {
            if let module::TargetManifestData::Beam(beam_start, beam_end) = target.data {
                target.ship.beam_hits(Some((beam_start, beam_end)), |module, _, _, hit| {
                    if let Some(hit_dist) = hit {
                        let hit_tick = 20 + (((3.0 - 1.0)*hit_dist*20.0) as u32);
                    
                        events.add(
                            hit_tick,
                            target.ship.index,
                            Box::new(DamageEvent::new(module.index, 1, 0, false)),
                        );
                    }
                });
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let mut sprite = SpriteSheet::new(asset_store.get_sprite_info_str("modules/small_beam_sprite.png"));

        if context.is_active {
            sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 1, 23, 0.2));
        } else {
            sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), sprite));
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        self.add_plan_effects(context, asset_store, effects);
        
        let ship_id = context.ship_id;
        
        if context.is_active {
            if let Some(ref target) = context.target {
                let target_ship_id = target.ship.id;
            
                if let module::TargetManifestData::Beam(beam_start, beam_end) = target.data {
                    let start_time = 1.0;
                    let end_time = 3.0;
                    
                    // Add the simulation visual for beam leaving ship screen
                    effects.add_visual(ship_id, 2, BeamExitVisual {
                        start_time: start_time,
                        end_time: end_time,
                        
                        beam_start: context.get_render_center() + Vec2 { x: 12.0, y: 0.0 },
                        
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
                    
                    effects.add_visual(target_ship_id, 2, beam_visual);
                    
                    effects.add_sound(start_time, 1, asset_store.get_sound(&"effects/beam1.ogg".to_string()).clone());
                }
            }
        }
    }
}
