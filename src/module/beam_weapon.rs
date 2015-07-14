use std::collections::HashMap;

#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::GlGraphics;

use battle_context::BattleContext;
use module;
use module::{IModule, Model, ModelIndex, Module, ModuleClass, ModuleContext, ModuleShape, TargetManifest, TargetManifestData};
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
pub struct BeamWeaponModule {
    damage: u8,

    old_rotation: f64,
    rotation: f64,
    
    base_sprite: Option<String>,
    turret_sprite: String,
    beam_mid: String,
    beam_end: String,
    
    turret_center: Vec2f,
    beam_pos: Vec2f,
    
    fire_anim_interval: (f64, f64),
}

impl BeamWeaponModule {
    pub fn new(model: ModelIndex) -> Module {
        Module::new(model, ModuleShape::new(vec![vec![b'#']]), 2, 2, 3,
            BeamWeaponModule {
                damage: 1,
            
                old_rotation: 0.0,
                rotation: 0.0,
                
                base_sprite: None,
                turret_sprite: "small_beam".to_string(),
                beam_mid: "small_beam_mid".to_string(),
                beam_end: "small_beam_end".to_string(),
                
                turret_center: Vec2::new(24.0, 24.0),
                beam_pos: Vec2::new(12.0, 0.0),
                
                fire_anim_interval: (1.0, 3.0),
            },
        )
    }
    
    pub fn from_properties(model: &Model, prop: &HashMap<String, String>) -> Module {
        let turret_center =
            match prop.get(&"turret_center_x".to_string()) {
                Some(ref turret_center_x) => {
                    Vec2::new(prop[&"turret_center_x".to_string()].parse().unwrap(),
                              prop[&"turret_center_y".to_string()].parse().unwrap())
                },
                None => { Vec2::new(0.0, 0.0) },
            };
    
        Module::from_model(model,
            BeamWeaponModule {
                damage: prop["damage"].parse().unwrap(),
            
                old_rotation: 0.0,
                rotation: 0.0,
                
                base_sprite: prop.get(&"base".to_string()).map(|s| s.clone()),
                turret_sprite: prop[&"turret".to_string()].clone(),
                beam_mid: prop[&"beam_mid".to_string()].clone(),
                beam_end: prop[&"beam_end".to_string()].clone(),
                
                turret_center: turret_center,
                beam_pos: Vec2::new(prop[&"beam_pos_x".to_string()].parse().unwrap(),
                                    prop[&"beam_pos_y".to_string()].parse().unwrap()),
                
                fire_anim_interval: (prop[&"fire_anim_start".to_string()].parse().unwrap(),
                                     prop[&"fire_anim_end".to_string()].parse().unwrap()),
            },
        )
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
                            Box::new(DamageEvent::new(module.index, self.damage, 0, false)),
                        );
                    }
                });
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        if let Some(ref base_sprite_name) = self.base_sprite {
            let mut base_sprite = SpriteSheet::new(asset_store.get_sprite_info(base_sprite_name));
            base_sprite.add_named_stay(&"idle".to_string(), 0.0, 7.0);
            effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), 0.0, base_sprite));
        }
        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.turret_sprite));
        
        weapon_sprite.center = self.turret_center;
        
        if context.is_active {
            weapon_sprite.add_named_stay(&"idle".to_string(), 0.0, 7.0);
        } else {
            weapon_sprite.add_named_stay(&"off".to_string(), 0.0, 7.0);
        }
        
        // Monolithic beam textures need beam above sprite.
        // Rotating turret head beams need beam below the turret, above the base.
        let layer =
            if self.base_sprite.is_some() {
                2
            } else {
                0
            };
        
        effects.add_visual(context.ship_id, layer, SpriteVisual::new(context.get_render_position() + weapon_sprite.center, self.rotation, weapon_sprite));
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        if let Some(ref base_sprite_name) = self.base_sprite {
            let mut base_sprite = SpriteSheet::new(asset_store.get_sprite_info(base_sprite_name));
            base_sprite.add_named_stay(&"idle".to_string(), 0.0, 7.0);
            effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), 0.0, base_sprite));
        }
        
        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.turret_sprite));
        weapon_sprite.center = self.turret_center;
        
        let ship_id = context.ship_id;
        
        if context.is_active {
            if let Some(ref target) = context.target {
                let target_ship_id = target.ship.id;
            
                if let module::TargetManifestData::Beam(beam_start, beam_end) = target.data {
                    let start_time = 1.0;
                    let end_time = 3.0;
                    
                    weapon_sprite.add_named_once(&"pre_fire".to_string(), 0.0, 1.0);
                    weapon_sprite.add_named_once(&"fire".to_string(), self.fire_anim_interval.0, self.fire_anim_interval.1);
                    weapon_sprite.add_named_stay(&"post_fire".to_string(), self.fire_anim_interval.1, 7.0);
                    
                    // Add the simulation visual for beam leaving ship screen
                    effects.add_visual(ship_id, 1, BeamExitVisual {
                        start_time: start_time,
                        end_time: end_time,
                        
                        beam_start: context.get_render_center() + self.beam_pos,
                        
                        texture: asset_store.get_texture(&self.beam_mid).clone(),
                    });
                    
                    // Add the simulation visual for beam entering target screen
                    let mut beam_end_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.beam_end));
                    beam_end_sprite.add_named_loop(&"loop".to_string(), 0.0, 2.0, 0.1); // 2 second beam duration
                    
                    let beam_visual =
                        BeamVisual::new(start_time, end_time,
                                        beam_start, beam_end,
                                        asset_store.get_texture(&self.beam_mid).clone(),
                                        beam_end_sprite);
                    
                    effects.add_visual(target_ship_id, 2, beam_visual);
                    
                    effects.add_sound(start_time, 1, asset_store.get_sound(&"effects/beam1.ogg".to_string()).clone());
                }
            } else {
                weapon_sprite.add_named_stay(&"idle".to_string(), 0.0, 7.0);
            }
        } else {
            weapon_sprite.add_named_stay(&"off".to_string(), 0.0, 7.0);
        }
        
        // Monolithic beam textures need beam above sprite.
        // Rotating turret head beams need beam below the turret, above the base.
        let layer =
            if self.base_sprite.is_some() {
                2
            } else {
                0
            };
        
        effects.add_visual(context.ship_id, layer, SpriteVisual::new(context.get_render_position() + weapon_sprite.center, self.rotation, weapon_sprite));
    }
}
