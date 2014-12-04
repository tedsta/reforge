use std::rc::Rc;

use event::{Events, GenericEvent, RenderArgs};
use graphics::Context;
use input::{keyboard, mouse, Keyboard, Mouse};
use opengl_graphics::{Gl, Texture};
use piston::Window;

use assets::GUI_TEXTURE;
use asset_store::AssetStore;
use battle_state::BattleContext;
use module::{IModule, MODULE_CATEGORIES, ModuleCategory, ModuleRef};
use net::ClientId;
use ship::{Ship, ShipRef};
use sim::SimVisuals;
use vec::{Vec2, Vec2f};

pub struct SpaceGui<'a> {
    module_category: Option<ModuleCategory>, // Selected module category
    
    // The target ships' render areas
    render_areas: Vec<ShipRenderArea>,
    
    // Selected module
    module: Option<ModuleRef>,
    
    mouse_x: f64,
    mouse_y: f64,
    
    // Textures
    category_textures: Vec<Rc<Texture>>,
}

impl<'a> SpaceGui<'a> {
    pub fn new(asset_store: &AssetStore, context: &BattleContext, my_client_id: ClientId) -> SpaceGui<'a> {
        let mut render_areas = vec!();
        for (client_id, ship) in context.ships_client_id.iter() {
            if *client_id != my_client_id {
                //let target = RenderTexture::new(500, 500, false).expect("Failed to create render texture");
                //let texture = target.get_texture().expect("Failed to get render texture's texture");
                render_areas.push(ShipRenderArea {
                    ship: Some(ship.clone()),
                    x: 772.0,
                    y: 8.0,
                    //target: target,
                    //texture: texture,
                });
                break;
            }
        }
    
        SpaceGui {
            module_category: None,
            render_areas: render_areas,
            module: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            
            category_textures: vec![
                asset_store.get_texture(GUI_TEXTURE).clone(),
                asset_store.get_texture(GUI_TEXTURE).clone(),
                asset_store.get_texture(GUI_TEXTURE).clone(),
                asset_store.get_texture(GUI_TEXTURE).clone(),
            ],
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, client_ship: &mut Ship) {
        use event::*;
    
        e.mouse_cursor(|x, y| {
            self.mouse_x = x;
            self.mouse_y = y;
        });
        e.press(|button| {
            match button {
                Keyboard(key) => self.on_key_pressed(key), 
                Mouse(button) => {
                    let (mouse_x, mouse_y) = (self.mouse_x, self.mouse_y);
                    match button {
                        mouse::Left => self.on_mouse_left_pressed(mouse_x, mouse_y, client_ship),
                        _ => {},
                    }
                },
            }
        });
    }
    
    pub fn reset_plan_stats(&mut self, client_ship: &mut Ship) {
        client_ship.state.plan_power = client_ship.state.power;
    }
    
    pub fn draw_planning(&mut self, r_args: &RenderArgs, gl: &mut Gl, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &Ship) {
        use graphics::*;
        
        let context = Context::abs(r_args.width as f64, r_args.height as f64);
        
        // Clear the screen
        context.rgb(0.0, 0.0, 0.0).draw(gl);
        
        // Draw player ship
        draw_ship(context.trans(150.0, 150.0), gl, sim_visuals, client_ship, 0.0);
    
        for render_area in self.render_areas.iter_mut() {
            // TODO clear render texture
        
            {
                let context = context.trans(render_area.x, render_area.y).trans(150.0, 150.0);
                
                draw_ship(context, gl, sim_visuals, render_area.ship.as_ref().unwrap().borrow().deref(), 0.0);
            }
            
            // TODO draw render texture
        }
    }
    
    pub fn draw_simulating(&mut self, r_args: &RenderArgs, gl: &mut Gl, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &Ship, time: f64) {
        use graphics::*;
        
        let context = Context::abs(r_args.width as f64, r_args.height as f64);
        
        // Clear the screen
        context.rgb(0.0, 0.0, 0.0).draw(gl);
        
        // Draw player ship
        draw_ship(context.trans(150.0, 150.0), gl, sim_visuals, client_ship, time);
    
        for render_area in self.render_areas.iter_mut() {
            // TODO clear render texture
        
            {
                let context = context.trans(render_area.x, render_area.y).trans(150.0, 150.0);
                
                draw_ship(context, gl, sim_visuals, render_area.ship.as_ref().unwrap().borrow().deref(), time);
            }
            
            // TODO draw render texture
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    fn on_mouse_left_pressed(&mut self, x: f64, y: f64, client_ship: &mut Ship) {
        if self.module.is_none() {
            let ship_offset_x = 150.0;
            let ship_offset_y = 150.0;
            let x = x - ship_offset_x;
            let y = y - ship_offset_y;

            for module in client_ship.modules.iter() {
                let mut module_borrowed = module.borrow_mut();
            
                // Get module position and size on screen
                let Vec2{x: module_x, y: module_y} = module_borrowed.get_base().get_render_position();
                let Vec2{x: module_w, y: module_h} = module_borrowed.get_base().get_render_size();
                let module_x = module_x as f64;
                let module_y = module_y as f64;
                let module_w = module_w as f64;
                let module_h = module_h as f64;
                if x >= module_x && x <= module_x+module_w && y >= module_y && y <= module_y+module_h {
                    let module_power = module_borrowed.get_base().get_power();
                    if module_borrowed.get_base().plan_powered {
                        self.module = Some(module.clone());
                    } else if module_borrowed.get_base().can_activate() && client_ship.state.plan_power >= module_power {
                        client_ship.state.plan_power -= module_power;
                        module_borrowed.get_base_mut().plan_powered = true;
                    }
                }
            }
        }
        
        for render_area in self.render_areas.iter() {
            let ship_offset_x = 150.0;
            let ship_offset_y = 150.0;
            let x = x - render_area.x - ship_offset_x;
            let y = y - render_area.y - ship_offset_y;
            match render_area.ship.as_ref() {
                Some(ship) => {
                    for module in ship.borrow().modules.iter() {
                        // Get module position and size on screen
                        let Vec2{x: module_x, y: module_y} = module.borrow().get_base().get_render_position();
                        let Vec2{x: module_w, y: module_h} = module.borrow().get_base().get_render_size();
                        let module_x = module_x as f64;
                        let module_y = module_y as f64;
                        let module_w = module_w as f64;
                        let module_h = module_h as f64;
                        if x >= module_x && x <= module_x+module_w && y >= module_y && y <= module_y+module_h {
                            if self.module.is_some() {
                                {
                                    // Inner scope for module ref so we can clear it after
                                    let selected_module = self.module.as_ref().unwrap();
                                    selected_module.borrow_mut().on_module_clicked(ship, module);
                                }
                                self.module = None;
                                return;
                            }
                        }
                    }
                },
                None => {},
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ShipRenderArea {
    ship: Option<ShipRef>,
    x: f64,
    y: f64,
    //target: RenderTexture,
    //texture: Texture,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn draw_ship(context: Context, gl: &mut Gl, sim_visuals: &mut SimVisuals, ship: &Ship, time: f64) {
    use graphics::*;

    sim_visuals.draw(&context, gl, ship.id, time);
    ship.draw_module_hp(&context, gl);
    for i in range(0, ship.state.get_hp()) {
        context
            .trans(-145.0, -145.0)
            .rect((i as f64)*18.0, 0.0, 16.0, 32.0)
            .rgb(0.0, 1.0, 0.0)
            .draw(gl);
    }
    
    for i in range(0, ship.state.shields) {
        context
            .trans(-145.0, -145.0 + 34.0)
            .rect((i as f64)*18.0, 0.0, 16.0, 32.0)
            .rgb(0.0, 0.0, 1.0)
            .draw(gl);
    }
    
    for i in range(0, ship.state.plan_power) {
        context
            .trans(-145.0, -145.0 + 68.0)
            .rect((i as f64)*18.0, 0.0, 16.0, 32.0)
            .rgb(1.0, 1.0, 0.0)
            .draw(gl);
    }
    
    for i in range(ship.state.plan_power, ship.state.power) {
        context
            .trans(-145.0, -145.0 + 68.0)
            .rect((i as f64)*18.0, 0.0, 16.0, 32.0)
            .rgba(1.0, 1.0, 0.0, 0.5)
            .draw(gl);
    }
}