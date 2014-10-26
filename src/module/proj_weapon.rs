use assets::LASER_TEXTURE;
use battle_state::TICKS_PER_SECOND;
use module::{IModule, Module, ModuleRef, ModuleBase, ProjectileWeapon, Weapon};
use net::{ClientId, InPacket, OutPacket};
use ship::{ShipId, ShipState};
use sim::SimEventAdder;
use vec::{Vec2, Vec2f};

#[cfg(client)]
use sim::SimVisuals;

#[deriving(Encodable, Decodable)]
pub struct ProjectileWeaponModule {
    pub base: ModuleBase,
    
    projectiles: Vec<Projectile>,
    
    target: Option<ModuleRef>,
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
        visuals.add(ship_id, |renderer, time| {
            renderer.draw_texture(LASER_TEXTURE, 1000.0*time, 300.0);
        });
    }
    
    fn after_simulation(&mut self, ship_state: &mut ShipState) {
    }
    
    /*fn draw(&mut self, renderer: &mut Renderer, context: &BattleContext, simulating: bool, time: f32) {
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
    }*/
    
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
    
    fn on_module_clicked(&mut self, module: &ModuleRef) -> bool {
        self.target = Some(module.clone());
        println!("Selected module");
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