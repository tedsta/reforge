use std::collections::HashMap;
use std::io::IoResult;

use module::{Module, ModuleBase, ProjectileWeapon};
use net::{ClientId, InPacket, OutPacket, Packable};
use render;
use render::{Renderer, TextureId};
use ship::Ship;
use sim_element::SimElement;

#[deriving(FromPrimitive)]
enum ProjectilePhase {
    OriginToOffscreen,
    OffscreenToTarget,
    Detonate
}

struct Projectile {
    texture: TextureId,
    phase: ProjectilePhase,
    damage: u8,
    hit: bool,
    
    fire_tick: u32,       // Tick that the projectile fires at
    offscreen_tick: u32, // Tick that the projectile starts travelling from offscreen to target at
    hit_tick: u32,        // Tick that projectile hits target at
}

pub struct ProjectileWeaponModule {
    pub base: ModuleBase,
    
    projectiles: Vec<Projectile>,
}

impl ProjectileWeaponModule {
    pub fn new() -> Module {
        ProjectileWeapon(ProjectileWeaponModule {
            base: ModuleBase::new(),
            projectiles: vec!(),
        })
    }
}

impl Packable for ProjectileWeaponModule {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<ProjectileWeaponModule> {
        let base = try!(packet.read());

        Ok(ProjectileWeaponModule {
            base: base,
            projectiles: vec!(),
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write(&self.base));
        Ok(())
    }
}

impl SimElement for ProjectileWeaponModule {
    fn server_preprocess(&mut self, ships: &HashMap<ClientId, Ship>) {
        for projectile in self.projectiles.iter_mut() {
            projectile.hit = true;
            projectile.fire_tick = 0;
            projectile.offscreen_tick = 20;
            projectile.hit_tick = 40;
        }
    }

    fn before_simulation(&mut self, ships: &HashMap<ClientId, Ship>) {
        for projectile in self.projectiles.iter_mut() {
            projectile.phase = OriginToOffscreen;
        }
    }
    
    fn on_simulation_time(&mut self, ships: &HashMap<ClientId, Ship>, time: u32) {
        println!("Simulating module at {}", time);
    }
    
    fn after_simulation(&mut self, ships: &HashMap<ClientId, Ship>) {
    }
    
    fn draw(&mut self, renderer: &mut Renderer, simulating: bool, time: f32) {
        renderer.draw_texture(render::Engine, (self.base.x as f32)*(48f32) + (time*100f32), (self.base.y as f32)*(48f32));
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