use std::cmp;
use std::iter::repeat;
use std::num::Float;
use std::ops::DerefMut;
use std::rand::Rng;
use std::rand;

#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

use battle_context::{BattleContext, tick_to_time};
use module;
use module::{IModule, Module, ModuleClass, ModuleContext, TargetManifest, TargetManifestData};
use net::{ClientId, InPacket, OutPacket};
use ship::{Ship, ShipId, ShipState};
use sim::SimEvents;
use sim_events::DamageEvent;
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim_visuals::{LerpVisual, SpriteVisual};
#[cfg(feature = "client")]
use sim::{SimEffects, SimVisual};
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct ProjectileWeaponModule {
    projectiles: Vec<Projectile>,
}

impl ProjectileWeaponModule {
    pub fn new() -> Module {
        let projectile = Projectile {
            damage: 1,
            hit: false,
        };
    
        Module::new(1, 1, 2, 2, 3,
            ProjectileWeaponModule {
                projectiles: repeat(projectile).take(3).collect(),
            }
        )
    }
}

impl IModule for ProjectileWeaponModule {
    fn get_class(&self) -> ModuleClass { ModuleClass::ProjectileWeapon }
    
    fn get_target_mode(&self) -> Option<module::TargetMode> {
        Some(module::TargetMode::TargetModule)
    }

    fn server_preprocess(&mut self, context: &ModuleContext) {    
        if let Some(ref target) = context.target {                
            // Random number generater
            let mut rng = rand::thread_rng();
            
            for projectile in self.projectiles.iter_mut() {
                if rng.gen::<f64>() > (0.15 * (cmp::min(target.ship.state.thrust, 5) as f64)) {
                    projectile.hit = true;
                } else {
                    projectile.hit = false;
                }
            }
        }
    }

    fn before_simulation(&mut self, context: &ModuleContext, events: &mut SimEvents) {
        if let Some(ref target) = context.target {
            if let module::TargetManifestData::TargetModule(ref target_module) = target.data {
                for (i, projectile) in self.projectiles.iter_mut().enumerate() {                                            
                    let start = (i*10) as u32;
                    
                    let hit_tick = start + 40;
                    
                    if projectile.hit {
                        events.add(
                            hit_tick,
                            target.ship.index,
                            Box::new(DamageEvent::new(target_module.index, 1)),
                        );
                    }
                }
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("modules/weapon_sprite.png"));
        
        if context.is_active {
            weapon_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 1));
        } else {
            weapon_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), weapon_sprite));
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let ship_id = context.ship_id;
    
        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("modules/weapon_sprite.png"));
        
        if context.is_active {
            if let Some(ref target) = context.target {
                let target_ship_id = target.ship.id;
            
                if let module::TargetManifestData::TargetModule(ref target_module) = target.data {                
                    let mut last_weapon_anim_end = 0.0;
                
                    for (i, projectile) in self.projectiles.iter().enumerate() {
                        use std::f64::consts::FRAC_PI_2;
                        
                        // Calculate positions
                        let fire_pos = context.get_render_center() + Vec2{x: 20.0, y: 0.0};
                        let to_offscreen_pos = fire_pos + Vec2{x: 1500.0, y: 0.0};
                        let from_offscreen_pos = Vec2{x: 1500.0, y: 0.0};
                        let hit_pos =
                            if projectile.hit {
                                target_module.get_render_center()
                            } else {
                                Vec2 { x: 200.0, y: 300.0 }
                            };
                        
                        // Calculate ticks
                        let start = (i*10) as u32;
                        let fire_tick = start;
                        let offscreen_tick = start + 20;
                        let hit_tick = start + 40;
                    
                        // Set up interpolation stuff to send projectile from weapon to offscreen
                        let start_time = tick_to_time(fire_tick);
                        let end_time = tick_to_time(offscreen_tick);
                        let start_pos = fire_pos;
                        let end_pos = to_offscreen_pos;
                        
                        let dist = end_pos - start_pos;
                        let rotation = dist.y.atan2(dist.x);
                        
                        let mut laser_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("effects/laser1.png"));
                        laser_sprite.centered = true;
                        laser_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 4, 0.05));
                        
                        let weapon_anim_start = start_time;
                        let weapon_anim_end = start_time+0.15;
                        
                        // Add weapon fire animations for this projectile
                        weapon_sprite.add_animation(SpriteAnimation::Stay(last_weapon_anim_end, weapon_anim_start, 1));
                        weapon_sprite.add_animation(SpriteAnimation::PlayOnce(weapon_anim_start, weapon_anim_end, 1, 6));
                        
                        // Set the last end for the next projectile
                        last_weapon_anim_end = weapon_anim_end;
                    
                        // Add the simulation visual for projectile leaving
                        effects.add_visual(ship_id, 2, LerpVisual {
                            start_time: start_time,
                            end_time: end_time,
                            start_pos: start_pos,
                            end_pos: end_pos,
                            start_rot: rotation,
                            end_rot: rotation,
                            sprite_sheet: laser_sprite,
                        });
                        
                        // Add the sound for projectile firing
                        effects.add_sound(start_time, 0, asset_store.get_sound(&"effects/laser.wav".to_string()).clone());
                        
                        // Set up interpolation stuff to send projectile from offscreen to target
                        let start_time = tick_to_time(offscreen_tick);
                        let end_time = tick_to_time(hit_tick);
                        let start_pos = from_offscreen_pos;
                        let end_pos = hit_pos;
                        
                        let dist = end_pos - start_pos;
                        let rotation = dist.y.atan2(dist.x);

                        let mut laser_sprite = SpriteSheet::new(asset_store.get_sprite_info_str("effects/laser1.png"));
                        laser_sprite.centered = true;
                        laser_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 4, 0.05));
                        
                        // Add the simulation visual for projectile entering target screen
                        effects.add_visual(target_ship_id, 2, LerpVisual {
                            start_time: start_time,
                            end_time: end_time,
                            start_pos: start_pos,
                            end_pos: end_pos,
                            start_rot: rotation,
                            end_rot: rotation,
                            sprite_sheet: laser_sprite,
                        });
                        
                        // Set up explosion visual
                        let start_time = tick_to_time(hit_tick);
                        let end_time = start_time + 0.7;
                        
                        let mut explosion_sprite =  SpriteSheet::new(asset_store.get_sprite_info_str("effects/explosion1.png"));
                        explosion_sprite.centered = true;
                        explosion_sprite.add_animation(SpriteAnimation::PlayOnce(start_time, end_time, 0, 9));
                        
                        effects.add_visual(target_ship_id, 3, SpriteVisual::new(hit_pos, explosion_sprite));
                        
                        // Add the sound for projectile exploding
                        effects.add_sound(start_time, 0, asset_store.get_sound(&"effects/small_explosion.wav".to_string()).clone());
                    }
                    
                    // Add last stay animation
                    weapon_sprite.add_animation(SpriteAnimation::Stay(last_weapon_anim_end, 7.0, 1));
                }
            } else {
                weapon_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 1));
            }
        } else {
            weapon_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
        
        effects.add_visual(ship_id, 0, SpriteVisual::new(context.get_render_position(), weapon_sprite));
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
        for projectile in self.projectiles.iter() {
            packet.write(&projectile.hit).unwrap();
        }
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
        for projectile in self.projectiles.iter_mut() {
            projectile.hit = packet.read().unwrap();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(RustcEncodable, RustcDecodable, Clone)]
struct Projectile {
    damage: u8,
    hit: bool,
}
