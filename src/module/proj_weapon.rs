use std::collections::HashMap;
use std::io::{IoResult, IoError, OtherIoError};

use battle_state::BattleContext;
use battle_state::TICKS_PER_SECOND;
use module::{Module, ModuleRef, ModuleBase, ProjectileWeapon};
use net::{ClientId, InPacket, OutPacket};
use render::{Renderer, TextureId, LASER_TEXTURE};
use sim_element::SimElement;
use vec::{Vec2, Vec2f};

#[deriving(Encodable, Decodable)]
pub struct ProjectileWeaponModule {
    pub base: ModuleBase,
    
    projectiles: Vec<Projectile>,
    
    target: Option<u32>,
}

impl ProjectileWeaponModule {
    pub fn new() -> Module {
        let projectile = Projectile {
            texture: LASER_TEXTURE,
            phase: FireToOffscreen,
            damage: 1,
            hit: false,
            
            fire_tick: 0,
            offscreen_tick: 0,
            hit_tick: 0,
            
            fire_pos: Vec2{x: 0f32, y: 0f32},
            to_offscreen_pos: Vec2{x: 0f32, y: 0f32},
            from_offscreen_pos: Vec2{x: 0f32, y: 0f32},
            hit_pos: Vec2{x: 0f32, y: 0f32},
        };
    
        ProjectileWeapon(ProjectileWeaponModule {
            base: ModuleBase::new(LASER_TEXTURE),
            projectiles: vec![projectile],
            target: None,
        })
    }
}

impl SimElement for ProjectileWeaponModule {
    fn server_preprocess(&mut self, context: &BattleContext) {
        for projectile in self.projectiles.iter_mut() {
            projectile.hit = true;
        }
    }

    fn before_simulation(&mut self, context: &BattleContext) {
        /*'ship: for ship in ships.values() {
            for module in ship.borrow().modules.iter() {
                self.target = Some(module.clone());
                break 'ship;
            }
        }
        let target_pos = self.target.as_ref().unwrap().borrow().deref().get_base().get_render_position();*/
    
        for projectile in self.projectiles.iter_mut() {
            projectile.phase = FireToOffscreen;
            projectile.fire_tick = 0;
            projectile.offscreen_tick = 20;
            projectile.hit_tick = 40;
            
            projectile.fire_pos = self.base.get_render_position();
            projectile.to_offscreen_pos = projectile.fire_pos + Vec2{x: 1500f32, y: 0f32};
            projectile.from_offscreen_pos = Vec2{x: 1500f32, y: 0f32};
            projectile.hit_pos = Vec2{x: 0f32, y: 0f32};
        }
    }
    
    fn on_simulation_time(&mut self, context: &BattleContext, tick: u32) {
        for projectile in self.projectiles.iter_mut() {
            match projectile.phase {
                FireToOffscreen => {
                    if tick >= projectile.offscreen_tick {
                        projectile.phase = OffscreenToTarget;
                    }
                },
                OffscreenToTarget => {
                    if tick >= projectile.hit_tick {
                        projectile.phase = Detonate;
                    }
                },
                Detonate => {
                },
            }
        }
    }
    
    fn after_simulation(&mut self, context: &BattleContext) {
    }
    
    fn draw(&mut self, renderer: &mut Renderer, context: &BattleContext, simulating: bool, time: f32) {
        let ship = self.base.get_ship(context);
        self.base.draw(renderer, ship);
        
        for projectile in self.projectiles.iter() {
            match projectile.phase {
                FireToOffscreen => {
                    let start_time = (projectile.fire_tick as f32)/(TICKS_PER_SECOND as f32);
                    let stop_time = (projectile.offscreen_tick as f32)/(TICKS_PER_SECOND as f32);
                    let interp = (time-start_time)/(stop_time-start_time);
                    ship.render_target.draw_texture_vec(renderer, projectile.texture, &(projectile.fire_pos + (projectile.to_offscreen_pos-projectile.fire_pos)*interp));
                },
                OffscreenToTarget => {
                    let start_time = (projectile.offscreen_tick as f32)/(TICKS_PER_SECOND as f32);
                    let stop_time = (projectile.hit_tick as f32)/(TICKS_PER_SECOND as f32);
                    let interp = (time-start_time)/(stop_time-start_time);
                    ship.render_target.draw_texture_vec(renderer, projectile.texture, &(projectile.from_offscreen_pos + (projectile.hit_pos-projectile.from_offscreen_pos)*interp));
                },
                Detonate => {
                },
            }
        }
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
    }
    
    fn read_plans(&self, packet: &mut InPacket) {
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
    }
    
    fn read_results(&self, packet: &mut InPacket) {
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Encodable, Decodable)]
enum ProjectilePhase {
    FireToOffscreen,
    OffscreenToTarget,
    Detonate
}

#[deriving(Encodable, Decodable)]
struct Projectile {
    texture: TextureId,
    phase: ProjectilePhase,
    damage: u8,
    hit: bool,
    
    // Simulation times that the projectile changes phases at
    fire_tick: u32,       // Tick that the projectile fires at
    offscreen_tick: u32,  // Tick that the projectile starts travelling from offscreen to target at
    hit_tick: u32,        // Tick that projectile hits target at
    
    // Render stuff

    // Interpolation points for drawing
    fire_pos: Vec2f,
    to_offscreen_pos: Vec2f,
    from_offscreen_pos: Vec2f,
    hit_pos: Vec2f,
}