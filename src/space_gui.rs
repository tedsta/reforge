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
        for (client_id, ship) in context.ships.iter() {
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
            
            category_textures: vec![asset_store.get_texture(GUI_TEXTURE).clone(), asset_store.get_texture(GUI_TEXTURE).clone(), asset_store.get_texture(GUI_TEXTURE).clone()],
        }
    }
    
    pub fn event<E: GenericEvent>(&mut self, e: &E, client_ship: &Ship) {
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
    
    pub fn draw_planning(&mut self, r_args: &RenderArgs, gl: &mut Gl, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &Ship) {
        use graphics::*;
        
        let context = Context::abs(r_args.width as f64, r_args.height as f64);
        
        // Clear the screen
        context.rgb(0.0, 0.0, 0.0).draw(gl);
        
        // Draw player ship
        let player_context = context.trans(100.0, 100.0);
        sim_visuals.draw(&player_context, gl, client_ship.id, 0.0);
        for i in range(0, client_ship.state.get_hp()) {
            player_context
                .trans(-95.0, -95.0)
                .rect((i as f64)*18.0, 0.0, 16.0, 32.0)
                .rgb(0.0, 1.0, 0.0)
                .draw(gl);
        }
    
        for render_area in self.render_areas.iter_mut() {
            // TODO clear render texture
        
            {
                let ship_context = context.trans(render_area.x, render_area.y).trans(100.0, 100.0);
                let ship = render_area.ship.as_ref().unwrap().borrow();
                
                sim_visuals.draw(&ship_context, gl, ship.id, 0.0);
                for i in range(0, ship.state.get_hp()) {
                    ship_context
                        .trans(-95.0, -95.0)
                        .rect((i as f64)*18.0, 0.0, 16.0, 32.0)
                        .rgb(0.0, 1.0, 0.0)
                        .draw(gl);
                }
            }
            
            // TODO draw render texture
        }
    
        self.draw_overlay(&context, gl, client_ship);
    }
    
    pub fn draw_simulating(&mut self, r_args: &RenderArgs, gl: &mut Gl, asset_store: &AssetStore, sim_visuals: &mut SimVisuals, client_ship: &Ship, time: f64) {
        use graphics::*;
        
        let context = Context::abs(r_args.width as f64, r_args.height as f64);
        
        // Clear the screen
        context.rgb(0.0, 0.0, 0.0).draw(gl);
        
        // Draw player ship
        let player_context = context.trans(100.0, 100.0);
        sim_visuals.draw(&player_context, gl, client_ship.id, time);
        for i in range(0, client_ship.state.get_hp()) {
            player_context
                .trans(-95.0, -95.0)
                .rect((i as f64)*18.0, 0.0, 16.0, 32.0)
                .rgb(0.0, 1.0, 0.0)
                .draw(gl);
        }
    
        for render_area in self.render_areas.iter_mut() {
            // TODO clear render texture
        
            {
                let ship_context = context.trans(render_area.x, render_area.y).trans(100.0, 100.0);
                let ship = render_area.ship.as_ref().unwrap().borrow();
                
                sim_visuals.draw(&ship_context, gl, ship.id, time);
                for i in range(0, ship.state.get_hp()) {
                    ship_context
                        .trans(-95.0, -95.0)
                        .rect((i as f64)*18.0, 0.0, 16.0, 32.0)
                        .rgb(0.0, 1.0, 0.0)
                        .draw(gl);
                }
            }
            
            // TODO draw render texture
        }
    
        self.draw_overlay(&context, gl, client_ship);
    }
    
    fn draw_overlay(&self, context: &Context, gl: &mut Gl, client_ship: &Ship) {
        use graphics::*;
    
        for category in MODULE_CATEGORIES.iter() {
            let icon_y: f64 =
                match self.module_category {
                    Some(c) if c == category.id => 584.0,
                    _ => { 600.0 },
                };
            
            context
                .image(self.category_textures[category.id as uint].deref())
                .trans(10.0 + (64.0*(category.id as u8 as f64)), icon_y)
                .draw(gl);
        }
        
        match self.module_category {
            Some(category) => {
                let mut i = 0u8;
                for module in client_ship.modules.iter() {
                    if module.borrow().get_base().category == category {
                        context
                            .image(self.category_textures[category as uint].deref())
                            .trans(10.0 + (64.0*(i as f64)), 500.0)
                            .draw(gl);
                        
                        i += 1;
                    }
                }
            },
            None => {},
        }
    }
    
    fn on_key_pressed(&mut self, key: keyboard::Key) {
    }
    
    fn on_mouse_left_pressed(&mut self, x: f64, y: f64, client_ship: &Ship) {
        if self.module.is_none() {
            match self.module_category {
                Some(category) => {
                    let mut i = 0u8;
                    for module in client_ship.modules.iter() {
                        if module.borrow_mut().get_base().category == category {
                            let icon_x = 10.0 + (64.0*(i as f64));
                            let icon_y: f64 = 500.0;
                            let icon_w = 48.0;
                            let icon_h = 48.0;
                            if x >= icon_x && x <= icon_x+icon_w && y >= icon_y && y <= icon_y+icon_h {
                                // If the player doesn't already have a selected module, and the module
                                // wants to be selected, select this module
                                if self.module.is_none() && module.borrow_mut().on_icon_clicked() {
                                    self.module = Some(module.clone());
                                    return;
                                }
                            }
                            i += 1;
                        }
                    }
                },
                None => {},
            }
        
            for category in MODULE_CATEGORIES.iter() {
                let icon_x = 10.0 + (64.0*(category.id as u32 as f64));
                let icon_y: f64 =
                    match self.module_category {
                        Some(c) if c == category.id => 584.0,
                        _ => { 600.0 },
                    };
                let icon_w = 48.0;
                let icon_h = 48.0;
                
                if x >= icon_x && x <= icon_x+icon_w && y >= icon_y && y <= icon_y+icon_h {
                    match self.module_category {
                        // Reclicked selected module category: deselect it
                        Some(c) if c == category.id => self.module_category = None,
                        // Selected a new module category
                        _ => self.module_category = Some(category.id),
                    }
                    return;
                }
            }
        }
        
        for render_area in self.render_areas.iter() {
            let ship_offset_x = 100.0;
            let ship_offset_y = 100.0;
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