use assets::{LASER_TEXTURE, TextureId};
use battle_state::TICKS_PER_SECOND;
use module::{IModule, Module, ModuleRef, ModuleBase, ProjectileWeapon, Weapon};
use net::{ClientId, InPacket, OutPacket};
use ship::{ShipId, ShipState};
use sim::SimEventAdder;
use vec::{Vec2, Vec2f};

#[cfg(client)]
use sim::{SimVisuals, SimVisual};
#[cfg(client)]
use sfml_renderer::SfmlRenderer;

#[deriving(Encodable, Decodable)]
pub struct ProjectileWeaponModule {
    pub base: ModuleBase,
    
    projectiles: Vec<Projectile>,
    
    target: Option<(ShipId, ModuleRef)>,
}

impl ProjectileWeaponModule {
    pub fn new() -> Module {
        let projectile = Projectile {
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
            base: ModuleBase::new(Weapon, LASER_TEXTURE),
            projectiles: vec![projectile],
            target: None,
        })
    }
}

impl IModule for ProjectileWeaponModule {
    fn server_preprocess(&mut self, ship_state: &mut ShipState) {
        for projectile in self.projectiles.iter_mut() {
            projectile.hit = true;
        }
    }

    fn before_simulation(&mut self, ship_state: &mut ShipState, events: &mut SimEventAdder) {
        events.add(20, |module| {
        });
    
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
    
    #[cfg(client)]
    fn add_sim_visuals(&self, ship_id: ShipId, visuals: &mut SimVisuals) {
        match self.target {
            Some((target_ship_id, ref target_module)) => {
                for projectile in self.projectiles.iter() {
                    // Set up interpolation stuff to send projectile from weapon to offscreen
                    let start_time = (projectile.fire_tick as f32)/(TICKS_PER_SECOND as f32);
                    let end_time = (projectile.offscreen_tick as f32)/(TICKS_PER_SECOND as f32);
                    let start_pos = projectile.fire_pos.clone();
                    let end_pos = projectile.to_offscreen_pos.clone();
                    let laser_texture = LASER_TEXTURE;
                
                    visuals.add(ship_id, box LerpVisual {
                        start_time: start_time,
                        end_time: end_time,
                        start_pos: start_pos,
                        end_pos: end_pos,
                        texture: laser_texture,
                    });
                    
                    // Set up interpolation stuff to send projectile from offscreen to target
                    let start_time = (projectile.offscreen_tick as f32)/(TICKS_PER_SECOND as f32);
                    let end_time = (projectile.hit_tick as f32)/(TICKS_PER_SECOND as f32);
                    let start_pos = projectile.from_offscreen_pos.clone();
                    let end_pos = projectile.hit_pos.clone();
                    let laser_texture = LASER_TEXTURE;
                    
                    visuals.add(target_ship_id, box LerpVisual {
                        start_time: start_time,
                        end_time: end_time,
                        start_pos: start_pos,
                        end_pos: end_pos,
                        texture: laser_texture,
                    });
                }
            },
            None => {},
        }
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
    }
    
    fn write_plans(&self, packet: &mut OutPacket) {
    }
    
    fn read_plans(&mut self, packet: &mut InPacket) {
    }
    
    fn write_results(&self, packet: &mut OutPacket) {
    }
    
    fn read_results(&mut self, packet: &mut InPacket) {
    }
    
    fn on_icon_clicked(&mut self) -> bool {
        println!("Clicked a weapon");
        true
    }
    
    fn on_module_clicked(&mut self, ship_id: ShipId, module: &ModuleRef) -> bool {
        self.target = Some((ship_id, module.clone()));
        println!("Targeted module");
        false
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

////////////////////////////////////////////////////////////////////////////////////////////////////

// Basic linear interpolation sim visual
#[cfg(client)]
pub struct LerpVisual {
    start_time: f32,
    end_time: f32,
    start_pos: Vec2f,
    end_pos: Vec2f,
    texture: TextureId,
}

#[cfg(client)]
impl SimVisual for LerpVisual {
    fn draw(&mut self, renderer: &SfmlRenderer, time: f32) {
        if time <= self.end_time {
            let interp = (time-self.start_time)/(self.end_time-self.start_time);
            renderer.draw_texture_vec(self.texture, &(self.start_pos + (self.end_pos-self.start_pos)*interp));
        }
    }
}