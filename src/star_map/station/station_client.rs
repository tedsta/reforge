use std::rc::Rc;
use std::cell::RefCell;

use ggez::{Context, GameResult, event::Event};

use asset_store::AssetStore;
//use chat::ChatGui;
use client_context::ReforgeClientContext;
use game_state::{self, GameState};
use module::{ModelIndex, ModelStore, ModuleStored};
use net::{Client, OutPacket};
use sector_data::SectorData;
use ship::ShipStored;
use sim::SimEffects;

use super::{ShipEditAction, StationAction, StationGui};

pub struct StationClient<'a> {
    time: f64,

    gui: StationGui,

    // The player's ship
    player_ship: Option<ShipStored>,

    sim_effects: SimEffects<'a>,
}

impl<'a> StationClient<'a> {
    pub fn new(
        gtx: &ReforgeClientContext,
        player_ship: Option<ShipStored>)
        //chat_gui: &mut ChatGui) -> GameResult<StationClient<'a>>
        -> GameResult<StationClient<'a>>
    {
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

        Ok(StationClient {
            time: 0.0,
            gui: StationGui::new(gtx, /*chat_gui, */module_inventory)?,
            player_ship: player_ship,
            sim_effects: SimEffects::new(),
        })
    }
    
    pub fn run(
        &mut self,
        gtx: &mut ReforgeClientContext, ctx: &mut Context) -> GameResult<()>
    {
        if let Some(ref ship) = self.player_ship {
            ship.add_simulation_effects(
                &mut gtx.asset_store, &mut gtx.model_store, &mut self.sim_effects);
        }

        match game_state::run(gtx, ctx, self)? {
            Some(gui_action) => {
                
            }
            _ => { },
        }
        Ok(())
    }
}

impl<'a> GameState for StationClient<'a> {
    type Context = ReforgeClientContext;
    type Action = ();

    fn event(&mut self, gtx: &mut Self::Context, e: &Event) -> Option<Self::Action> {
        if let Some(gui_action) = self.gui.event(gtx, &e, &self.player_ship) {
            let mut packet = OutPacket::new();
            packet.write(&gui_action);
            gtx.client.send(&packet);
            
            match gui_action {
                StationAction::Jump(_) => {
                    // Don't have to do anything because we just sent the action to the server
                    // Have to exit state by returning Some(_) though
                    return Some(());
                },
                StationAction::ShipEdit(ship_edit) => {
                    if let Some(ref mut ship) = self.player_ship {
                        match ship_edit {
                            ShipEditAction::Place(model, x, y) => {
                                let mut module = ModuleStored::from_module(
                                    model.get(&gtx.model_store).create());
                                module.x = x;
                                module.y = y;
                                
                                ship.add_module(module);
                                
                                self.sim_effects.reset();
                                ship.add_simulation_effects(&gtx.asset_store, &gtx.model_store, &mut self.sim_effects);
                            },
                            ShipEditAction::Remove(module) => {
                                // TD 2018-12-9: TODO?
                            },
                        }
                    }
                },
                StationAction::Chat(_) => { },
                StationAction::Logout => {
                    // Don't have to do anything because we just sent the action to the server
                    // Have to exit state by returning Some(_) though
                    return Some(());
                },
            }
        }

        None
    }

    fn draw(&mut self, gtx: &mut Self::Context, ctx: &mut Context) -> GameResult<()> {
        if let Ok(mut packet) = gtx.client.try_receive() {
            //let chat_msg = packet.read().unwrap();
            //gui.chat_gui.add_message(chat_msg);
        }

        let dt = 1.0 / 60.0;
        self.time += dt;
        //self.time += (1.0/60.0) + args.ext_dt;
        if self.time > 5.0 {
            self.time -= 5.0;
            
            if let Some(ref ship) = self.player_ship {
                self.sim_effects.reset();
                ship.add_simulation_effects(
                    &gtx.asset_store, &gtx.model_store, &mut self.sim_effects);
            }
        }
    
        self.gui.draw(
            gtx, ctx,
            &mut self.sim_effects,
            &self.player_ship,
            self.time, dt)?;

        Ok(())
    }
}
