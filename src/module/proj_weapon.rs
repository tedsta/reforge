use std::cmp;
use std::old_io::File;
use std::iter::repeat;
use std::num::Float;
use std::ops::DerefMut;
use std::rand::Rng;
use std::rand;

#[cfg(feature = "client")]
use graphics::Context;
#[cfg(feature = "client")]
use opengl_graphics::Gl;

use assets::{WEAPON_TEXTURE, LASER_TEXTURE, EXPLOSION_TEXTURE, TextureId};
use battle_state::{BattleContext, TICKS_PER_SECOND};
use module;
use module::{IModule, Module, ModuleRef, ModuleBase, ModuleBox};
use net::{ClientId, InPacket, OutPacket};
use ship::{ShipId, ShipRef, ShipState};
use sim::{SimEvent, SimEventAdder};
use vec::{Vec2, Vec2f};

#[cfg(feature = "client")]
use sim::{SimVisuals, SimVisual};
#[cfg(feature = "client")]
use sprite_sheet::{SpriteSheet, SpriteAnimation};
#[cfg(feature = "client")]
use asset_store::AssetStore;

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub struct ProjectileWeaponModule {
    projectiles: Vec<Projectile>,
}

impl ProjectileWeaponModule {
    pub fn new() -> Module<ProjectileWeaponModule> {
        let projectile = Projectile {
            damage: 1,
            hit: false,
            
            fire_tick: 0,
            offscreen_tick: 0,
            hit_tick: 0,
            
            fire_pos: Vec2{x: 0f64, y: 0f64},
            to_offscreen_pos: Vec2{x: 0f64, y: 0f64},
            from_offscreen_pos: Vec2{x: 0f64, y: 0f64},
            hit_pos: Vec2{x: 0f64, y: 0f64},
        };
    
        Module {
            base: ModuleBase::new(1, 1, 2, 2, 3),
            module: ProjectileWeaponModule {
                projectiles: repeat(projectile).take(3).collect(),
            },
        }
    }
    
    pub fn from_file(path: &Path) -> Module<ProjectileWeaponModule> {
        let mut damage = 1;
        let mut num_shots = 1;
        let mut texture = String::from_str("WEAPON");
    
        if let Ok(mut file) = File::open(path) {
            let contents = file.read_to_end();
        } else {
            panic!("Failed to read projectile weapon file");
        }
    
        let projectile = Projectile {
            damage: damage,
            hit: false,
            
            fire_tick: 0,
            offscreen_tick: 0,
            hit_tick: 0,
            
            fire_pos: Vec2{x: 0f64, y: 0f64},
            to_offscreen_pos: Vec2{x: 0f64, y: 0f64},
            from_offscreen_pos: Vec2{x: 0f64, y: 0f64},
            hit_pos: Vec2{x: 0f64, y: 0f64},
        };
    
        Module {
            base: ModuleBase::new(1, 1, 2, 2, 3),
            module: ProjectileWeaponModule {
                projectiles: repeat(projectile).take(3).collect(),
            },
        }
    }
}

impl IModule for ProjectileWeaponModule {
    fn server_preprocess(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {    
        if base.powered {
            if let Some(module::TargetData::TargetModule(ref target_ship, ref target_module)) = base.target_data {
                // Random number generater
                let mut rng = rand::thread_rng();
                
                let target_ship = target_ship.borrow();
                
                for projectile in self.projectiles.iter_mut() {
                    if rng.gen::<f64>() > (0.15 * (cmp::min(target_ship.state.thrust, 5) as f64)) {
                        projectile.hit = true;
                    } else {
                        projectile.hit = false;
                    }
                }
            }
        }
    }

    fn before_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, events: &mut SimEventAdder) {
        if base.powered {
            if let Some(module::TargetData::TargetModule(ref target_ship, ref target_module)) = base.target_data {
                for (i, projectile) in self.projectiles.iter_mut().enumerate() {                                            
                    let start = (i*10) as u32;
                    
                    projectile.fire_tick = start;
                    projectile.offscreen_tick = start + 20;
                    projectile.hit_tick = start + 40;
                    
                    projectile.fire_pos = base.get_render_center() + Vec2{x: 20.0, y: 0.0};
                    projectile.to_offscreen_pos = projectile.fire_pos + Vec2{x: 1500.0, y: 0.0};
                    projectile.from_offscreen_pos = Vec2{x: 1500.0, y: 0.0};
                    
                    if projectile.hit {
                        projectile.hit_pos = target_module.borrow().get_base().get_render_center();
                    
                        events.add(projectile.hit_tick, box DamageEvent::new(target_ship.clone(), target_module.clone(), 1));
                    } else {
                        projectile.hit_pos = Vec2{x: 200.0, y: 300.0};
                    }
                }
            }
        }
    }
    
    #[cfg(feature = "client")]
    fn add_plan_visuals(&self, base: &ModuleBase, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info(WEAPON_TEXTURE));
        
        if base.is_active() {
            weapon_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 1));
        } else {
            weapon_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
    
        visuals.add(ship.borrow().id, 0, box SpriteVisual {
            position: base.get_render_position().clone(),
            sprite_sheet: weapon_sprite,
        });
    }
    
    #[cfg(feature = "client")]
    fn add_simulation_visuals(&self, base: &ModuleBase, asset_store: &AssetStore, visuals: &mut SimVisuals, ship: &ShipRef) {
        let ship_id = ship.borrow().id;
    
        let mut weapon_sprite = SpriteSheet::new(asset_store.get_sprite_info(WEAPON_TEXTURE));
        
        if base.powered {
            if let Some(module::TargetData::TargetModule(ref target_ship, ref target_module)) = base.target_data {
                let target_ship_id = target_ship.borrow().id;
            
                let mut last_weapon_anim_end = 0.0;
            
                for projectile in self.projectiles.iter() {
                    use std::f64::consts::FRAC_PI_2;
                
                    // Set up interpolation stuff to send projectile from weapon to offscreen
                    let start_time = (projectile.fire_tick as f64)/(TICKS_PER_SECOND as f64);
                    let end_time = (projectile.offscreen_tick as f64)/(TICKS_PER_SECOND as f64);
                    let start_pos = projectile.fire_pos.clone();
                    let end_pos = projectile.to_offscreen_pos.clone();
                    
                    let dist = end_pos - start_pos;
                    let rotation = dist.y.atan2(dist.x);
                    
                    let mut laser_sprite = SpriteSheet::new(asset_store.get_sprite_info(LASER_TEXTURE));
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
                    visuals.add(ship_id, 1, box LerpVisual {
                        start_time: start_time,
                        end_time: end_time,
                        start_pos: start_pos,
                        end_pos: end_pos,
                        start_rot: rotation,
                        end_rot: rotation,
                        sprite_sheet: laser_sprite,
                    });
                    
                    // Set up interpolation stuff to send projectile from offscreen to target
                    let start_time = (projectile.offscreen_tick as f64)/(TICKS_PER_SECOND as f64);
                    let end_time = (projectile.hit_tick as f64)/(TICKS_PER_SECOND as f64);
                    let start_pos = projectile.from_offscreen_pos.clone();
                    let end_pos = projectile.hit_pos.clone();
                    
                    let dist = end_pos - start_pos;
                    let rotation = dist.y.atan2(dist.x);

                    let mut laser_sprite = SpriteSheet::new(asset_store.get_sprite_info(LASER_TEXTURE));
                    laser_sprite.centered = true;
                    laser_sprite.add_animation(SpriteAnimation::Loop(0.0, 7.0, 0, 4, 0.05));
                    
                    // Add the simulation visual for projectile entering target screen
                    visuals.add(target_ship_id, 1, box LerpVisual {
                        start_time: start_time,
                        end_time: end_time,
                        start_pos: start_pos,
                        end_pos: end_pos,
                        start_rot: rotation,
                        end_rot: rotation,
                        sprite_sheet: laser_sprite,
                    });
                    
                    // Set up explosion visual
                    let start_time = (projectile.hit_tick as f64)/(TICKS_PER_SECOND as f64);
                    let end_time = start_time + 0.7;
                    
                    let mut explosion_sprite =  SpriteSheet::new(asset_store.get_sprite_info(EXPLOSION_TEXTURE));
                    explosion_sprite.centered = true;
                    explosion_sprite.add_animation(SpriteAnimation::PlayOnce(start_time, end_time, 0, 9));
                    
                    visuals.add(target_ship_id, 1, box SpriteVisual {
                        position: projectile.hit_pos.clone(),
                        sprite_sheet: explosion_sprite,
                    });
                }
                
                // Add last stay animation
                weapon_sprite.add_animation(SpriteAnimation::Stay(last_weapon_anim_end, 7.0, 1));
            } else {
                weapon_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 1));
            }
        } else {
            weapon_sprite.add_animation(SpriteAnimation::Stay(0.0, 7.0, 0));
        }
        
        visuals.add(ship_id, 0, box SpriteVisual {
            position: base.get_render_position().clone(),
            sprite_sheet: weapon_sprite,
        });
    }
    
    fn after_simulation(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState) {
    }
    
    fn on_ship_removed(&mut self, base: &mut ModuleBase, ship_id: ShipId) {
    }
    
    fn write_results(&self, base: &ModuleBase, packet: &mut OutPacket) {
        for projectile in self.projectiles.iter() {
            packet.write(&projectile.hit).unwrap();
        }
    }
    
    fn read_results(&mut self, base: &mut ModuleBase, packet: &mut InPacket) {
        for projectile in self.projectiles.iter_mut() {
            projectile.hit = packet.read().unwrap();
        }
    }
    
    fn on_activated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
    }
    
    fn on_deactivated(&mut self, base: &mut ModuleBase, ship_state: &mut ShipState, modules: &Vec<ModuleRef>) {
    }
    
    fn get_target_mode(&self, base: &ModuleBase) -> Option<module::TargetMode> {
        Some(module::TargetMode::TargetModule)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(RustcEncodable, RustcDecodable, Clone)]
struct Projectile {
    damage: u8,
    hit: bool,
    
    // Simulation times that the projectile changes phases at
    fire_tick: u32,       // Tick that the projectile fires at
    offscreen_tick: u32,  // Tick that the projectile starts travelling from offscreen to target at
    hit_tick: u32,        // Tick that projectile hits target at
    
    // Interpolation points for drawing
    fire_pos: Vec2f,
    to_offscreen_pos: Vec2f,
    from_offscreen_pos: Vec2f,
    hit_pos: Vec2f,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DamageEvent {
    ship: ShipRef,
    module: ModuleRef,
    damage: u8,
}

impl DamageEvent {
    pub fn new(ship: ShipRef, module: ModuleRef, damage: u8) -> DamageEvent {
        DamageEvent {
            ship: ship,
            module: module,
            damage: damage,
        }
    }
}

impl SimEvent for DamageEvent {
    fn apply(&mut self, module: &mut ModuleBox) {
        let mut ship = self.ship.borrow_mut();
        ship.deal_damage(self.module.borrow_mut().deref_mut(), self.damage);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Basic linear interpolation sim visual
#[cfg(feature = "client")]
pub struct LerpVisual {
    start_time: f64,
    end_time: f64,
    start_pos: Vec2f,
    end_pos: Vec2f,
    start_rot: f64,
    end_rot: f64,
    sprite_sheet: SpriteSheet,
}

#[cfg(feature = "client")]
impl SimVisual for LerpVisual {
    fn draw(&mut self, context: &Context, gl: &mut Gl, time: f64) {
        if time >= self.start_time && time <= self.end_time {
            let interp = (time-self.start_time)/(self.end_time-self.start_time);
            let pos = self.start_pos + (self.end_pos-self.start_pos)*interp;
            let rot = self.start_rot + (self.start_rot-self.end_rot)*interp;
            self.sprite_sheet.draw(context, gl, pos.x, pos.y, rot, time);
        }
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
