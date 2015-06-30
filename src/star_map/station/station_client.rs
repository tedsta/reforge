use std::rc::Rc;
use std::cell::RefCell;

use piston::event::Events;
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use glutin_window::GlutinWindow;

use asset_store::AssetStore;
use chat::ChatGui;
use module::{ModelIndex, ModelStore};
use net::{Client, OutPacket};
use sector_data::SectorData;
use ship::ShipStored;
use sim::SimEffects;

use super::{ShipEditAction, StationAction, StationGui};

pub struct StationClient<'a> {
    client: &'a mut Client,
    
    // The player's ship
    player_ship: Option<ShipStored>,
}

impl<'a> StationClient<'a> {
    pub fn new(client: &'a mut Client, player_ship: Option<ShipStored>) -> StationClient<'a> {
        StationClient {
            client: client,
            player_ship: player_ship,
        }
    }
    
    pub fn run(&mut self,
               window: &Rc<RefCell<GlutinWindow>>,
               gl: &mut GlGraphics,
               glyph_cache: &mut GlyphCache,
               asset_store: &AssetStore,
               model_store: &ModelStore,
               chat_gui: &mut ChatGui,
               sectors: Vec<SectorData>) {     
        let module_inventory =
            vec![
                ("engine".to_string(), vec![(ModelIndex(0), 100)]),
                ("command".to_string(), vec![(ModelIndex(1), 100), (ModelIndex(7), 100)]),
                ("power".to_string(), vec![(ModelIndex(2), 100)]),
                ("shields".to_string(), vec![(ModelIndex(3), 100)]),
                ("weapons".to_string(), vec![(ModelIndex(4), 100), (ModelIndex(5), 100),
                                             (ModelIndex(6), 100), (ModelIndex(8), 100),
                                             (ModelIndex(9), 100)]),
            ];
    
        let ref mut gui = StationGui::new(model_store, chat_gui, sectors, module_inventory);
        let ref mut sim_effects = SimEffects::new();
        
        if let Some(ref ship) = self.player_ship {
            ship.add_simulation_effects(asset_store, model_store, sim_effects);
        }
    
        let mut time: f64 = 0.0;
        for e in Events::events(window.clone()) {
            use piston::event;
            use piston::input;
            use piston::event::*;

            let e: event::Event<input::Input> = e;
        
            // Forward events to GUI
            let gui_action = gui.event(&e, &self.player_ship);
            
            // Render GUI
            e.render(|args: &RenderArgs| {
                gl.draw(args.viewport(), |c, gl| {
                    time += (1.0/60.0) + args.ext_dt;
                    if time > 5.0 {
                        time -= 5.0;
                        
                        if let Some(ref ship) = self.player_ship {
                            sim_effects.reset();
                            ship.add_simulation_effects(asset_store, model_store, sim_effects);
                        }
                    }
                
                    gui.draw(
                        &c,
                        gl,
                        glyph_cache,
                        asset_store,
                        sim_effects,
                        &self.player_ship,
                        time,
                        (1.0/60.0) + args.ext_dt,
                    );
                });
            });
            
            if let Ok(mut packet) = self.client.try_receive() {
                let chat_msg = packet.read().unwrap();
                gui.chat_gui.add_message(chat_msg);
            }
            
            // Handle GUI action
            if let Some(gui_action) = gui_action {
                let mut packet = OutPacket::new();
                packet.write(&gui_action);
                self.client.send(&packet);
                
                match gui_action {
                    StationAction::Jump(_) => {
                        return;
                    },
                    StationAction::ShipEdit(ship_edit) => {
                        if let Some(ref mut ship) = self.player_ship {
                            match ship_edit {
                                ShipEditAction::Place(model, x, y) => {
                                    let mut module = model.get(model_store).create();
                                    module.x = x;
                                    module.y = y;
                                    
                                    ship.add_module(module);
                                    
                                    sim_effects.reset();
                                    ship.add_simulation_effects(asset_store, model_store, sim_effects);
                                },
                                ShipEditAction::Remove(module) => {
                                },
                            }
                        }
                    },
                    StationAction::Chat(_) => { },
                    StationAction::Logout => {
                        return;
                    },
                }
            }
        }
    }
}
