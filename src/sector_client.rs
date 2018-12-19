use std::cell::RefCell;
use std::cmp;
use std::io;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use time;

use ggez::{Context, GameResult, event::Event};

use asset_store::AssetStore;
use battle_context::{BattleContext, TICKS_PER_SECOND};
use client_context::ReforgeClientContext;
//use chat::ChatGui;
use game_state::{self, GameState};
use module::ModelStore;
use net::{Client, InPacket, OutPacket};
use packet_types::{ClientBattlePacket, ServerBattlePacket};
use sector_data::SectorData;
use ship::{Ship, ShipId, ShipIndex};
use sim::{SimEvents, SimEffects};
use space_gui::{SpaceGui, SpaceGuiAction};

pub struct ClientBattleState<'a> {
    // Context holding all the things involved in this battle
    bc: BattleContext,
    
    // The player's ship
    player_ship: ShipIndex,
    
    new_ships_pre: Option<InPacket>,
    results: Option<InPacket>,
    new_ships_post: Option<InPacket>,
    
    final_ticks: Option<u8>,

    gui: SpaceGui,
    sim_effects: SimEffects<'a>,
    sim_events: SimEvents<'a>,

    start_time: time::Timespec,
    elapsed_seconds: f64,
    next_tick: u32,
    plans_sent: bool,
}

impl<'a> ClientBattleState<'a> {
    pub fn new(
        gtx: &mut ReforgeClientContext, bc: BattleContext,
        ctx: &mut Context)
        -> GameResult<Self>
    {
        let player_ship = bc.get_ship_by_client_id(gtx.client.get_id()).index;
        let gui = SpaceGui::new(gtx, &bc, ctx, /*chat_gui,*/ player_ship)?;
        Ok(ClientBattleState {
            bc: bc,
            player_ship: player_ship,
            new_ships_pre: None,
            results: None,
            new_ships_post: None,
            final_ticks: None,

            gui: gui,
            sim_effects: SimEffects::new(),
            sim_events: SimEvents::new(),

            start_time: time::now().to_timespec(),
            elapsed_seconds: 0.0,
            next_tick: 0,
            plans_sent: false,
        })
    }
    
    pub fn run(
        &mut self,
        gtx: &mut ReforgeClientContext,
        ctx: &mut Context,
        //chat_gui: &mut ChatGui,
        server_results_sent: bool)
        -> GameResult<()>
    {
        // TODO display joining screen here
        
        if server_results_sent {
            // Wait for the tick
            loop {
                // We might get chat packets here
                let tick_packet = gtx.client.receive();
                let ticked = self.handle_packet(tick_packet);
                
                if ticked {
                    break;
                }
            }
        }
        
        // Get first turn's results
        loop {
            // Loop until tick packet is received
            let packet = gtx.client.receive();
            let ticked = self.handle_packet(packet);
            if ticked {
                break;
            }
        }

        loop {
            ////////////////////////////////
            // Simulate
            
            if let Some(ref mut ticks_left) = self.final_ticks {
                if *ticks_left == 0 {
                    break;
                } else {
                    *ticks_left -= 1;
                }
            }
            
            // Receive simulation results
            let mut new_ships_pre = self.new_ships_pre.take().expect("New ships pre packet must exist here");
            //let mut results = self.results.take().expect("Results packet must exist here");
            let mut new_ships_post = self.new_ships_post.take().expect("New ships post packet must exist here");
            
            self.handle_new_ships_packet(&mut new_ships_pre);
            if let Some(mut results) = self.results.take() {
                self.handle_simulation_results(&mut results);
            }
            
            let continue_state = self.run_simulation_phase(gtx, ctx)?;
            
            // Receive ships after sim
            self.handle_new_ships_packet(&mut new_ships_post);

            if !continue_state { break; }
        }

        Ok(())
    }
    
    fn run_simulation_phase(&mut self,
                            gtx: &mut ReforgeClientContext,
                            ctx: &mut Context)
                            -> GameResult<bool>
    {
        // Unlock any exploding or jumping ships
        let ships_to_unlock: Vec<ShipIndex> =
            self.bc.ships_iter()
                .filter_map(|s| if s.jumping || s.exploding { Some(s.index) } else { None })
                .collect();
        
        for ship_index in ships_to_unlock {
            self.bc.on_ship_removed(ship_index);
            self.gui.plans.on_ship_removed(ship_index);
        }

        self.sim_events = SimEvents::new();
        self.plans_sent = false;

        // Before simulation
        self.sim_effects.reset();
        self.bc.before_simulation(&gtx.model_store, &mut self.sim_events);
        self.bc.add_simulation_effects(&gtx.asset_store, &gtx.model_store, &mut self.sim_effects);
        
        // Simulation
        self.start_time = time::now().to_timespec();

        // Run the simulation phase
        let continue_state = match game_state::run(gtx, ctx, self)? {
            Some(action) => { true }
            _ => { false },
        };
        
        // Simulate any remaining ticks
        for t in self.next_tick .. 100 {
            self.sim_events.apply_tick(&mut self.bc, t);
        }
        
        // After simulation
        self.bc.after_simulation();
        
        // Apply module stats
        self.bc.apply_module_stats();
        
        // Deactivate modules that can no longer be powered
        self.bc.deactivate_unpowerable_modules();
        
        // Set all the dead ships to exploding
        for ship in self.bc.ships_iter_mut() {
            if ship.state.get_hp() == 0 {
                ship.exploding = true;
            }
        }

        println!("Finish simulation phase");

        Ok(continue_state)
    }
    
    fn build_plans_packet(&mut self) -> OutPacket {
        self.player_ship.get_mut(&mut self.bc).next_waypoint = self.gui.plans.next_waypoint;
        self.gui.set_next_waypoint();
        let mut packet = OutPacket::new();
        packet.write(&ServerBattlePacket::Plan).unwrap();
        packet.write(&self.gui.plans).unwrap();
        packet
    }
    
    fn send_chat(&mut self, client: &mut Client, msg: String) {
        let mut packet = OutPacket::new();
        packet.write(&ServerBattlePacket::Chat(msg)).unwrap();
        client.send(&packet);
    }
    
    fn send_logout(&mut self, client: &mut Client) {
        let mut packet = OutPacket::new();
        packet.write(&ServerBattlePacket::Logout).unwrap();
        client.send(&packet);
    }
    
    fn handle_packet(&mut self, mut packet: InPacket) -> bool {
        let battle_packet: ClientBattlePacket = packet.read().unwrap();

        println!("Handling a packet");
        
        match battle_packet {
            ClientBattlePacket::NewShipsPre => {
                println!("Got new ships pre packet");
                self.new_ships_pre = Some(packet);
            },
            ClientBattlePacket::SimResults => {
                println!("Got results packet");
                self.results = Some(packet);
            },
            ClientBattlePacket::NewShipsPost => {
                println!("Got new ships post packet");
                self.new_ships_post = Some(packet);
            },
            ClientBattlePacket::Tick(final_ticks) => {
                println!("Got tick packet");
                self.final_ticks = final_ticks;
                return true;
            },
            ClientBattlePacket::Chat(msg) => {
                println!("Got chat packet");
                //self.gui.chat_gui.add_message(msg);
            },
        }
        
        false
    }
    
    fn handle_simulation_results(&mut self, packet: &mut InPacket) {
        // Results packet has both plans and results
        self.bc.read_results(packet);
    }
    
    fn handle_new_ships_packet(&mut self, packet: &mut InPacket) {
        let ships_to_add: Vec<Ship> = packet.read().ok().expect("Failed to read ships to add from packet");
        let ships_to_remove: Vec<ShipIndex> = packet.read().ok().expect("Failed to read ships to remove from packet");
        
        let player_ship_id = self.player_ship.get(&self.bc).id;
        let player_hp = self.player_ship.get(&self.bc).state.get_hp();
        
        for ship in ships_to_remove.into_iter() {
            println!("Removing ship {:?}", ship);
            
            self.gui.on_ship_removed(self.player_ship, ship);
            self.bc.remove_ship(ship);
        }
    
        for ship in ships_to_add.into_iter() {
            println!("Got a new ship {:?}", ship.id);
            let ship_id = ship.id;
            if ship_id == player_ship_id {
                if player_hp == 0 {
                    println!("Replacing player's ship");
                    self.player_ship = ship.index;
                    self.gui.set_client_ship(&ship);
                    self.bc.add_ship(ship);
                }
            } else {
                println!("Trying to lock");
                let ship_index = ship.index;
                self.bc.add_ship(ship);
                self.gui.try_lock(ship_index);
            }
            println!("Added the ship");
        }

        println!("Finished readng new ships");
    }
}

impl<'a> GameState for ClientBattleState<'a> {
    type Context = ReforgeClientContext;
    type Action = ();

    fn event(&mut self, gtx: &mut Self::Context, e: &Event) -> Option<Self::Action> {
        // Forward events to GUI
        let gui_action = self.gui.event(gtx, &mut self.bc, e, self.player_ship, self.elapsed_seconds);
        
        if let Some(gui_action) = gui_action {
            match gui_action {
                SpaceGuiAction::Chat(msg) => {
                    self.send_chat(&mut gtx.client, msg);
                    None // Stay in this state
                },
                SpaceGuiAction::Logout => {
                    self.send_logout(&mut gtx.client);
                    //Some(()) // Leave this state?
                    None
                },
            }
        } else {
            None
        }
    }

    fn update(&mut self, gtx: &mut Self::Context) -> Option<Self::Action> {
        // Calculate a bunch of time stuff
        let current_time = time::now().to_timespec();
        let elapsed_time = current_time - self.start_time;
        self.elapsed_seconds = (elapsed_time.num_milliseconds() as f64)/1000.0;

        if !self.final_ticks.is_some() && !self.player_ship.get(&self.bc).exploding && !self.plans_sent && self.elapsed_seconds >= 2.5 {
            // Send plans
            let packet = self.build_plans_packet();
            gtx.client.send(&packet);
            self.plans_sent = true;
            println!("Sent plans at {}", self.elapsed_seconds);
        }
        
        if !self.final_ticks.is_some() {
            if self.plans_sent || self.player_ship.get(&self.bc).exploding {
                if let Ok(packet) = gtx.client.try_receive() {
                    let ticked = self.handle_packet(packet);
                    
                    if ticked && !self.final_ticks.is_some() {
                        // If the tick we got isn't the last tick, this turn is done.
                        println!("Finished turn at {}", self.elapsed_seconds);
                        return Some(());
                    }
                }
            }
        } else if self.elapsed_seconds >= 5.0 {
            println!("Finished turn because we're leaving this state");
            return Some(());
        }
        
        // Calculate current tick
        let tick = (elapsed_time.num_milliseconds() as u32)/(1000/TICKS_PER_SECOND);
        
        // Simulate any new ticks
        if self.next_tick < 100 {
            for t in self.next_tick .. cmp::min(self.next_tick+tick-self.next_tick+1, 100) {
                self.sim_events.apply_tick(&mut self.bc, t);
            }
            self.next_tick = tick+1;
        }

        None
    }

    fn draw(&mut self, gtx: &mut Self::Context, ctx: &mut Context) -> GameResult<()> {
        let dt = 1.0 / 60.0;
        self.gui.draw_simulating(
            gtx, &self.bc, ctx, &mut self.sim_effects,
            self.player_ship.get(&self.bc), self.elapsed_seconds,
            dt)?;

        Ok(())
    }
}
