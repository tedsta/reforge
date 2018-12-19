use std::cmp;
use std::collections::HashMap;
use std::iter::repeat;
use num::Float;
use std::ops::DerefMut;
use rand::Rng;
use rand;

use battle_context::{BattleContext, tick_to_time};
use module;
use module::{IModule, Model, ModelIndex, Module, ModuleClass, ModuleContext, ModuleShape, TargetManifest, TargetManifestData};
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectileWeaponModule {
    old_rotation: f64,
    rotation: f64,
    next_rotation: f64,
    projectiles: Vec<Projectile>,
    
    base_sprite: String,
    turret_sprite: String,
    projectile_sprite: String,
    explosion_sprite: String,
    
    turret_center: Vec2f,
}

impl ProjectileWeaponModule {
    pub fn new(model: ModelIndex) -> Module {
        let projectile = Projectile {
            damage: 1,
            hit: false,
        };
    
        Module::new(model, ModuleShape::new(vec![vec![b'#']]), 2, 2, 3,
            ProjectileWeaponModule {
                old_rotation: 0.0,
                rotation: 0.0,
                next_rotation: 0.0,
                projectiles: repeat(projectile).take(3).collect(),
                
                base_sprite: "pewpewbase".to_string(),
                turret_sprite: "pewpewfire".to_string(),
                projectile_sprite: "laser2".to_string(),
                explosion_sprite: "explosion1".to_string(),
                
                turret_center: Vec2::new(18.0, 24.0),
            },
        )
    }
    
    pub fn from_properties(model: &Model, prop: &HashMap<String, String>) -> Module {
        let projectile = Projectile {
            damage: prop["projectile_damage"].parse().unwrap(),
            hit: false,
        };
        
        let turret_center =
            match prop.get(&"turret_center_x".to_string()) {
                Some(ref turret_center_x) => {
                    Vec2::new(prop[&"turret_center_x".to_string()].parse().unwrap(),
                              prop[&"turret_center_y".to_string()].parse().unwrap())
                },
                None => { Vec2::new(0.0, 0.0) },
            };
    
        Module::from_model(model,
            ProjectileWeaponModule {
                old_rotation: 0.0,
                rotation: 0.0,
                next_rotation: 0.0,
                projectiles: repeat(projectile).take(prop["num_projectiles"].parse().unwrap()).collect(),
                
                base_sprite: prop[&"base".to_string()].clone(),
                turret_sprite: prop[&"turret".to_string()].clone(),
                projectile_sprite: prop[&"projectile".to_string()].clone(),
                explosion_sprite: prop[&"explosion".to_string()].clone(),
                
                turret_center: turret_center,
            },
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
        use std::f64::consts::PI;
    
        let mut rng = rand::thread_rng();
        
        self.old_rotation = self.next_rotation;
    
        if let Some(ref target) = context.target {
            if let module::TargetManifestData::TargetModule(ref target_module) = target.data {
                let target_move_vector = target.ship.lerp_next_waypoint(tick_to_time(10)) -
                                         context.ship_lerp_next_waypoint(tick_to_time(10));
                self.rotation = f64::atan2(-target_move_vector.y, target_move_vector.x);

                let target_move_vector = target.ship.lerp_next_waypoint(tick_to_time(100)) -
                                         context.ship_lerp_next_waypoint(tick_to_time(100));
                self.next_rotation = f64::atan2(-target_move_vector.y, target_move_vector.x);
            
                for (i, projectile) in self.projectiles.iter_mut().enumerate() {                                            
                    let start = (i*10) as u32;
                    
                    let hit_tick = start + 40;
                    
                    if projectile.hit {
                        events.add(
                            hit_tick,
                            target.ship.index,
                            Box::new(DamageEvent::new(target_module.index, 1, 0, true)),
                        );
                    }
                }
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_plan_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let mut base_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.base_sprite));
        base_sprite.add_named_stay(&"idle".to_string(), 0.0, 7.0);
        effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), 0.0, base_sprite));

        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.turret_sprite));
        
        weapon_sprite.center = self.turret_center;
        
        if context.is_active {
            weapon_sprite.add_named_stay(&"idle".to_string(), 0.0, 7.0);
        } else {
            weapon_sprite.add_named_stay(&"off".to_string(), 0.0, 7.0);
        }
        
        effects.add_visual(context.ship_id, 2, SpriteVisual::new(context.get_render_position() + weapon_sprite.center, self.rotation, weapon_sprite));
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_effects(&self, context: &ModuleContext, asset_store: &AssetStore, effects: &mut SimEffects) {
        let ship_id = context.ship_id;
        
        // Add rotating lerp visual
        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.turret_sprite));
        weapon_sprite.center = self.turret_center;
        weapon_sprite.add_named_stay(&"idle".to_string(), 0.0, tick_to_time(10));
        effects.add_visual(ship_id, 2,
            LerpVisual {
                start_time: 0.0,
                end_time: tick_to_time(10),
                start_pos: context.get_render_position() + weapon_sprite.center,
                end_pos: context.get_render_position() + weapon_sprite.center,
                start_rot: self.rotation,
                end_rot: self.rotation,
                sprite_sheet: weapon_sprite,
            });
    
        // Base sprite animation
        let mut base_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.base_sprite));
        base_sprite.add_named_stay(&"idle".to_string(), 0.0, 7.0);
        effects.add_visual(context.ship_id, 0, SpriteVisual::new(context.get_render_position(), 0.0, base_sprite));
        
        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.turret_sprite));
        weapon_sprite.center = self.turret_center;

        let mut weapon_sprite_end_rotation = self.rotation;
        
        if context.is_active {
            if let Some(ref target) = context.target {
                let target_ship_id = target.ship.id;
            
                if let module::TargetManifestData::TargetModule(ref target_module) = target.data {                
                    let mut last_weapon_anim_end = 0.0;
                
                    for (i, projectile) in self.projectiles.iter().enumerate() {
                        use std::f64::consts::FRAC_PI_2;
                        
                        // Calculate positions
                        let fire_pos = context.get_render_center() + Vec2::new(30.0, 0.0).rotate(self.rotation);
                        let to_offscreen_pos = fire_pos + Vec2::new(1500.0, 0.0).rotate(self.rotation);
                        let from_offscreen_pos = Vec2{x: 1500.0, y: 0.0};
                        let hit_pos =
                            if projectile.hit {
                                target_module.get_render_center()
                            } else {
                                Vec2 { x: 200.0, y: 300.0 }
                            };
                        
                        // Calculate ticks
                        let start = (i*10) as u32 + 10;
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
                        
                        let mut laser_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.projectile_sprite));
                        laser_sprite.center();
                        laser_sprite.add_named_loop(&"loop".to_string(), 0.0, 7.0, 0.05);
                        
                        let weapon_anim_start = start_time;
                        let weapon_anim_end = start_time+0.15;
                        
                        // Add weapon fire animations for this projectile
                        if i != 0 {
                            weapon_sprite.add_named_stay(&"idle".to_string(), last_weapon_anim_end, weapon_anim_start);
                        }
                        weapon_sprite.add_named_once(&"fire".to_string(), weapon_anim_start, weapon_anim_end);
                        
                        // Set the last end for the next projectile
                        last_weapon_anim_end = weapon_anim_end;
                    
                        // Add the simulation visual for projectile leaving
                        effects.add_visual(ship_id, 3, LerpVisual {
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

                        let mut laser_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.projectile_sprite));
                        laser_sprite.center();
                        laser_sprite.add_named_loop(&"loop".to_string(), 0.0, 7.0, 0.05);
                        
                        // Add the simulation visual for projectile entering target screen
                        effects.add_visual(target_ship_id, 3, LerpVisual {
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
                        
                        let mut explosion_sprite =  SpriteSheet::new(asset_store.get_sprite_info(&self.explosion_sprite));
                        explosion_sprite.center();
                        explosion_sprite.add_named_once(&"explode".to_string(), start_time, end_time);
                        
                        effects.add_visual(target_ship_id, 4, SpriteVisual::new(hit_pos, 0.0, explosion_sprite));
                        
                        // Add the sound for projectile exploding
                        effects.add_sound(start_time, 0, asset_store.get_sound(&"effects/small_explosion.wav".to_string()).clone());
                    }
                    
                    // Add last stay animation
                    weapon_sprite.add_named_stay(&"idle".to_string(), last_weapon_anim_end, 5.0);

                    let end_aim_dir = target.ship.lerp_next_waypoint(5.0) -
                                      context.ship_lerp_next_waypoint(5.0);
                    let end_rotation = f64::atan2(-end_aim_dir.y, end_aim_dir.x);
                    effects.add_visual(ship_id, 2, 
                                       LerpVisual {
                                           start_time: tick_to_time(10),
                                           end_time: 5.0,
                                           start_pos: context.get_render_position() + weapon_sprite.center,
                                           end_pos: context.get_render_position() + weapon_sprite.center,
                                           start_rot: self.rotation,
                                           end_rot: end_rotation,
                                           sprite_sheet: weapon_sprite,
                                       });

                    let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info(&self.turret_sprite));
                    weapon_sprite.add_named_stay(&"idle".to_string(), 5.0, 7.0);
                    weapon_sprite.center = self.turret_center;
                    effects.add_visual(ship_id, 2, 
                                       SpriteVisual::new(context.get_render_position() + weapon_sprite.center,
                                                         self.rotation, weapon_sprite));
                }
            } else {
                weapon_sprite.add_named_stay(&"idle".to_string(), 0.0, 7.0);
                effects.add_visual(ship_id, 2, 
                                   SpriteVisual::new(context.get_render_position() + weapon_sprite.center,
                                                     self.rotation, weapon_sprite));
            }
        } else {
            weapon_sprite.add_named_stay(&"off".to_string(), 0.0, 7.0);
            effects.add_visual(ship_id, 2, 
                               SpriteVisual::new(context.get_render_position() + weapon_sprite.center,
                                                 self.rotation, weapon_sprite));
        }
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

#[derive(Serialize, Deserialize, Clone)]
struct Projectile {
    damage: u8,
    hit: bool,
}
