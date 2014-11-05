use assets::{WEAPON_TEXTURE, LASER_TEXTURE, EXPLOSION_TEXTURE, TextureId};
use battle_state::TICKS_PER_SECOND;
use module::{IModule, Module, ModuleRef, ModuleBase, ModuleType, ModuleTypeStore, ProjectileWeapon, Weapon};
use net::{ClientId, InPacket, OutPacket};
use ship::{ShipId, ShipState};
use sim::SimEventAdder;
use vec::{Vec2, Vec2f};

#[cfg(client)]
use sim::{SimVisuals, SimVisual};
#[cfg(client)]
use sfml_renderer::SfmlRenderer;
#[cfg(client)]
use sprite_sheet::{SpriteSheet, Loop, PlayOnce};
#[cfg(client)]
use asset_store::AssetStore;

#[deriving(Encodable, Decodable)]
pub struct ProjectileWeaponModule {
    pub base: ModuleBase,
    
    projectiles: Vec<Projectile>,
    
    target: Option<(ShipId, ModuleRef)>,
}

impl ProjectileWeaponModule {
    pub fn new(mod_store: &ModuleTypeStore, mod_type: ModuleType) -> Module {
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
            base: ModuleBase::new(mod_store, mod_type),
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
    
        match self.target {
            Some((_, ref target_module)) => {
                for projectile in self.projectiles.iter_mut() {
                    projectile.phase = FireToOffscreen;
                    projectile.fire_tick = 0;
                    projectile.offscreen_tick = 20;
                    projectile.hit_tick = 40;
                    
                    projectile.fire_pos = self.base.get_render_position();
                    projectile.to_offscreen_pos = projectile.fire_pos + Vec2{x: 1500.0, y: 0.0};
                    projectile.from_offscreen_pos = Vec2{x: 1500.0, y: 0.0};
                    projectile.hit_pos = target_module.borrow().get_base().get_render_position();
                }
            }
            None => { },
        }
    }
    
    #[cfg(client)]
    fn add_plan_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship_id: ShipId) {
        let mut weapon_sprite =  SpriteSheet::new(asset_store.get_sprite_info(WEAPON_TEXTURE));
        weapon_sprite.add_animation(Loop(0.0, 5.0, 0, 5, 0.05));
    
        visuals.add(ship_id, box SpriteVisual {
            position: self.base.get_render_position().clone(),
            sprite_sheet: weapon_sprite,
        });
    }
    
    #[cfg(client)]
    fn add_simulation_visuals(&self, asset_store: &AssetStore, visuals: &mut SimVisuals, ship_id: ShipId) {
        self.add_plan_visuals(asset_store, visuals, ship_id);
    
        match self.target {
            Some((target_ship_id, ref target_module)) => {
                for projectile in self.projectiles.iter() {
                    // Set up interpolation stuff to send projectile from weapon to offscreen
                    let start_time = (projectile.fire_tick as f32)/(TICKS_PER_SECOND as f32);
                    let end_time = (projectile.offscreen_tick as f32)/(TICKS_PER_SECOND as f32);
                    let start_pos = projectile.fire_pos.clone();
                    let end_pos = projectile.to_offscreen_pos.clone();
                    
                    let mut laser_sprite = SpriteSheet::new(asset_store.get_sprite_info(LASER_TEXTURE));
                    laser_sprite.add_animation(Loop(0.0, 5.0, 0, 4, 0.05));
                
                    // Add the simulation visual for projectile leaving
                    visuals.add(ship_id, box LerpVisual {
                        start_time: start_time,
                        end_time: end_time,
                        start_pos: start_pos,
                        end_pos: end_pos,
                        sprite_sheet: laser_sprite,
                    });
                    
                    // Set up interpolation stuff to send projectile from offscreen to target
                    let start_time = (projectile.offscreen_tick as f32)/(TICKS_PER_SECOND as f32);
                    let end_time = (projectile.hit_tick as f32)/(TICKS_PER_SECOND as f32);
                    let start_pos = projectile.from_offscreen_pos.clone();
                    let end_pos = projectile.hit_pos.clone();

                    let mut laser_sprite = SpriteSheet::new(asset_store.get_sprite_info(LASER_TEXTURE));
                    laser_sprite.add_animation(Loop(0.0, 5.0, 0, 4, 0.05));
                    
                    // Add the simulation visual for projectile entering target screen
                    visuals.add(target_ship_id, box LerpVisual {
                        start_time: start_time,
                        end_time: end_time,
                        start_pos: start_pos,
                        end_pos: end_pos,
                        sprite_sheet: laser_sprite,
                    });
                    
                    // Set up explosion visual
                    let start_time = (projectile.hit_tick as f32)/(TICKS_PER_SECOND as f32);
                    let end_time = start_time + 0.7;
                    
                    let mut explosion_sprite =  SpriteSheet::new(asset_store.get_sprite_info(EXPLOSION_TEXTURE));
                    explosion_sprite.add_animation(PlayOnce(start_time, end_time, 0, 10));
                    
                    visuals.add(target_ship_id, box SpriteVisual {
                        position: projectile.hit_pos.clone(),
                        sprite_sheet: explosion_sprite,
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
    sprite_sheet: SpriteSheet,
}

#[cfg(client)]
impl SimVisual for LerpVisual {
    fn draw(&mut self, renderer: &SfmlRenderer, time: f32) {
        if time <= self.end_time {
            let interp = (time-self.start_time)/(self.end_time-self.start_time);
            let pos = self.start_pos + (self.end_pos-self.start_pos)*interp;
            self.sprite_sheet.draw(renderer, pos.x, pos.y, time);
        }
    }
}

// Sprite sheet sim visual
#[cfg(client)]
pub struct SpriteVisual {
    position: Vec2f,
    sprite_sheet: SpriteSheet,
}

#[cfg(client)]
impl SimVisual for SpriteVisual {
    fn draw(&mut self, renderer: &SfmlRenderer, time: f32) {
        self.sprite_sheet.draw(renderer, self.position.x, self.position.y, time);
    }
}