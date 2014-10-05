use std::collections::HashMap;
use std::io::{IoResult, IoError, OtherIoError};

use battle_state::TICKS_PER_SECOND;
use module::{Module, ModuleBase, ProjectileWeapon};
use net::{ClientId, InPacket, OutPacket, Packable};
use render::{Renderer, TextureId, LASER_TEXTURE};
use ship::ShipRef;
use sim_element::SimElement;
use vec::{Vec2, Vec2f};

pub struct ProjectileWeaponModule {
    pub base: ModuleBase,
    
    projectiles: Vec<Projectile>,
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
            detonate_tick: 0,
            
            fire_pos: Vec2{x: 0f32, y: 0f32},
            to_offscreen_pos: Vec2{x: 0f32, y: 0f32},
            from_offscreen_pos: Vec2{x: 0f32, y: 0f32},
            target_pos: Vec2{x: 0f32, y: 0f32},
        };
    
        ProjectileWeapon(ProjectileWeaponModule {
            base: ModuleBase::new(LASER_TEXTURE),
            projectiles: vec![projectile],
        })
    }
}

impl Packable for ProjectileWeaponModule {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<ProjectileWeaponModule> {
        let base = try!(packet.read());
        let projectiles = try!(packet.read_vec());

        Ok(ProjectileWeaponModule {
            base: base,
            projectiles: projectiles,
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write(&self.base));
        try!(packet.write_vec(&self.projectiles));
        Ok(())
    }
}

impl SimElement for ProjectileWeaponModule {
    fn server_preprocess(&mut self, ships: &HashMap<ClientId, ShipRef>) {
        for projectile in self.projectiles.iter_mut() {
            projectile.hit = true;
        }
    }

    fn before_simulation(&mut self, ships: &HashMap<ClientId, ShipRef>) {
        for projectile in self.projectiles.iter_mut() {
            projectile.phase = FireToOffscreen;
            projectile.fire_tick = 0;
            projectile.offscreen_tick = 20;
            projectile.detonate_tick = 40;
            
            projectile.fire_pos = Vec2{x: 0f32, y: 200f32};
            projectile.to_offscreen_pos = Vec2{x: 1500f32, y: 200f32};
        }
    }
    
    fn on_simulation_time(&mut self, ships: &HashMap<ClientId, ShipRef>, tick: u32) {
        for projectile in self.projectiles.iter_mut() {
            match projectile.phase {
                FireToOffscreen => {
                    if tick >= projectile.offscreen_tick {
                        projectile.phase = OffscreenToTarget;
                    }
                },
                OffscreenToTarget => {
                    if tick >= projectile.detonate_tick {
                        projectile.phase = Detonate;
                    }
                },
                Detonate => {
                },
            }
        }
    }
    
    fn after_simulation(&mut self, ships: &HashMap<ClientId, ShipRef>) {
    }
    
    fn draw(&mut self, renderer: &mut Renderer, simulating: bool, time: f32) {
        self.base.draw(renderer);
        
        for projectile in self.projectiles.iter() {
            match projectile.phase {
                FireToOffscreen => {
                    let start_time = (projectile.fire_tick as f32)/(TICKS_PER_SECOND as f32);
                    let duration = (projectile.offscreen_tick as f32)/(TICKS_PER_SECOND as f32);
                    let interp = (time-start_time)/duration;
                    renderer.draw_texture_vec(projectile.texture, &(projectile.fire_pos + (projectile.to_offscreen_pos-projectile.fire_pos)*interp));
                },
                OffscreenToTarget => {
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

#[deriving(FromPrimitive)]
enum ProjectilePhase {
    FireToOffscreen,
    OffscreenToTarget,
    Detonate
}

struct Projectile {
    texture: TextureId,
    phase: ProjectilePhase,
    damage: u8,
    hit: bool,
    
    // Simulation times that the projectile changes phases at
    fire_tick: u32,       // Tick that the projectile fires at
    offscreen_tick: u32,  // Tick that the projectile starts travelling from offscreen to target at
    detonate_tick: u32,        // Tick that projectile hits target at
    
    // Render stuff

    // Interpolation points for drawing
    fire_pos: Vec2f,
    to_offscreen_pos: Vec2f,
    from_offscreen_pos: Vec2f,
    target_pos: Vec2f,
}

impl Packable for Projectile {
    fn read_from_packet(packet: &mut InPacket) -> IoResult<Projectile> {
        let texture = try!(packet.read_u16());
        let phase = 
            match FromPrimitive::from_u8(try!(packet.read_u8())) {
                Some(phase) => phase,
                None => return Err(IoError{kind: OtherIoError, desc: "Failed to read projectile phase from packet", detail: None})
            };
        let damage = try!(packet.read_u8());
        let hit = try!(packet.read_bool());
        
        let fire_tick = try!(packet.read_u32());
        let offscreen_tick = try!(packet.read_u32());
        let detonate_tick = try!(packet.read_u32());
        
        let fire_pos = Vec2{x: try!(packet.read_f32()), y: try!(packet.read_f32())};
        let to_offscreen_pos = Vec2{x: try!(packet.read_f32()), y: try!(packet.read_f32())};
        let from_offscreen_pos = Vec2{x: try!(packet.read_f32()), y: try!(packet.read_f32())};
        let target_pos = Vec2{x: try!(packet.read_f32()), y: try!(packet.read_f32())};
    
        Ok(Projectile {
            texture: texture,
            phase: phase,
            damage: damage,
            hit: hit,
            
            fire_tick: fire_tick,
            offscreen_tick: offscreen_tick,
            detonate_tick: detonate_tick,

            fire_pos: fire_pos,
            to_offscreen_pos: to_offscreen_pos,
            from_offscreen_pos: from_offscreen_pos,
            target_pos: target_pos,
        })
    }
    
    fn write_to_packet(&self, packet: &mut OutPacket) -> IoResult<()> {
        try!(packet.write_u16(self.texture));
        try!(packet.write_u8(self.phase as u8));
        try!(packet.write_u8(self.damage));
        try!(packet.write_bool(self.hit));
        
        try!(packet.write_u32(self.fire_tick));
        try!(packet.write_u32(self.offscreen_tick));
        try!(packet.write_u32(self.detonate_tick));
        
        try!(packet.write_f32(self.fire_pos.x)); try!(packet.write_f32(self.fire_pos.y));
        try!(packet.write_f32(self.to_offscreen_pos.x)); try!(packet.write_f32(self.to_offscreen_pos.y));
        try!(packet.write_f32(self.from_offscreen_pos.x)); try!(packet.write_f32(self.from_offscreen_pos.y));
        try!(packet.write_f32(self.target_pos.x)); try!(packet.write_f32(self.target_pos.y));
        Ok(())
    }
}