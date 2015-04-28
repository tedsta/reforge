use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{Sender, Receiver};
use time;

use ai::run_ai;
use battle_context::{BattleContext, ClientPacketId, ServerPacketId};
use login::AccountBox;
use module::Module;
use net::{ClientId, ServerSlot, ServerSlotId, SlotInMsg, InPacket, OutPacket};
use ship::{Ship, ShipId, ShipIndex, ShipPlans, ShipStored};
use sim::SimEvents;

pub struct SectorState {
    slot: ServerSlot,
    star_map_slot_id: ServerSlotId,

    // Context holding all the things involved in this battle
    context: BattleContext,
    
    turn_start_time: time::Timespec,
    simulated_turn: bool,
    
    received_plans: HashSet<ClientId>,
    clients_waiting: HashSet<ClientId>,
    clients_active: HashSet<ClientId>,
    
    // Client plans list
    ship_plans: Vec<(ShipIndex, ShipPlans)>,
    
    // All the clients' accounts
    accounts: HashMap<ClientId, AccountBox>,
    
    // Ships to add after simulation
    ships_to_add: Vec<ShipIndex>,
    
    // Ships to remove after simulation
    ships_to_remove: Vec<ShipIndex>,
    
    turn_number: u32,
    
    debug: bool,
}

impl SectorState {
    pub fn new(slot: ServerSlot, star_map_slot_id: ServerSlotId, context: BattleContext, debug: bool) -> SectorState {
        SectorState {
            slot: slot,
            star_map_slot_id: star_map_slot_id,
            context: context,
            turn_start_time: time::now().to_timespec(),
            simulated_turn: false,
            received_plans: HashSet::new(),
            clients_waiting: HashSet::new(),
            clients_active: HashSet::new(),
            ship_plans: vec!(),
            accounts: HashMap::new(),
            ships_to_add: vec!(),
            ships_to_remove: vec!(),
            turn_number: 0,
            debug: debug,
        }
    }
    
    pub fn run(&mut self, to_map_sender: Sender<AccountBox>, from_map_receiver: Receiver<AccountBox>, ack: Sender<()>, create_ai: bool) {
        if create_ai {
            // TODO: come up with better way to generate AI ship IDs
            let ai_ship = Ship::generate((100000000) as ShipId, "n00bslayer808".to_string(), 2);
            self.context.add_ship(ai_ship);
            
            let ai_ship = Ship::generate((100000001) as ShipId, "thing1".to_string(), 2);
            self.context.add_ship(ai_ship);
            
            let ai_ship = Ship::generate((100000002) as ShipId, "thing2".to_string(), 2);
            self.context.add_ship(ai_ship);
            
            let ai_ship = Ship::generate((100000003) as ShipId, "daisy_girl".to_string(), 2);
            self.context.add_ship(ai_ship);
        }
    
        loop {
            ///////////////////////////////////////////////////////////
            // Check if it's time to simulate next turn
            
            // Get the current time from our turn timer
            let turn_time = time::now().to_timespec() - self.turn_start_time;
            
            if !self.simulated_turn && turn_time.num_milliseconds() >= 3500 {
                self.simulate_next_turn(&to_map_sender);

                self.simulated_turn = true;
            }
            
            if turn_time.num_milliseconds() >= 5000 {
                // Reset the turn stuff
                self.simulated_turn = false;
                self.turn_start_time = time::now().to_timespec();
                
                self.send_turn_tick();
            }
        
            ///////////////////////////////////////////////////////////
            // Receiver ServerSlot messages
            if let Ok(msg) = self.slot.try_receive() {
                match msg {
                    SlotInMsg::Joined(client_id) => {
                        println!("Client {} joined battle {}", client_id, self.slot.get_id());
                    },
                    SlotInMsg::ReceivedPacket(client_id, mut packet) => { self.handle_packet(client_id, &mut packet); },
                    _ => {}
                }
            }
            
            ///////////////////////////////////////////////////////////
            // Receive new clients
            if let Ok(mut account) = from_map_receiver.try_recv() {
                if self.debug {
                    println!("Receiving account");
                }
                println!("Receiving account {}", self.simulated_turn);
                let client_id = account.client_id.expect("This must have a client ID");
                
                // Add the client to the waiting list
                self.clients_waiting.insert(client_id);
                
                // Get the ship out of storage
                let ship_stored = account.ship.take().expect("This account must have a ship");
                let ship = ship_stored.to_ship(Some(client_id));
                
                // Add the player's account
                self.accounts.insert(client_id, account);
                
                // Send initial join packet
                let mut packet = OutPacket::new();
                packet.write(&ship).unwrap();
                packet.write(&self.simulated_turn).unwrap(); // Whether or not to start at simulation instead of planning phase
                packet.write(&self.context.ships).unwrap();
                self.slot.send(client_id, packet);
                
                // Add the player's ship
                let ship_index = self.context.add_ship(ship);
                self.ships_to_add.push(ship_index);
                
                ack.send(());
            }
        }
    }
    
    fn handle_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
        let id: ServerPacketId = match packet.read() {
            Ok(id) => id,
            Err(e) => {
                println!("Received invalid packet from client {}: {}", client_id, e);
                return;
            }
        };
        
        match id {
            ServerPacketId::Plan => {
                if self.debug {
                    println!("Handling plans packet");
                }
            
                self.received_plans.insert(client_id);
                
                // Handle the plans
                self.handle_plans_packet(client_id, packet);
                
                println!("Received plans packet from {} for turn {}", client_id, self.turn_number);
 
                if self.received_plans == self.clients_active {
                    // TODO decide if we should simulate turn a little early here.
                }
            },
        }
    }
    
    fn handle_plans_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
        let ship = self.context.get_ship_by_client_id(client_id);

        let plans = packet.read().ok().expect("Failed to receive client's plans");
        
        if !ship.exploding {
            // Don't save these plans if the ship is exploding
            self.ship_plans.push((ship.index, plans));
        }
    }
    
    fn simulate_next_turn(&mut self, to_map_sender: &Sender<AccountBox>) {
        if self.debug {
            println!("Simulating next turn");
        }
    
        // Send new ships to added/removed before simulation
        self.send_new_ships();
    
        // Run AI on ships with no client
        for ship in self.context.ships_iter() {
            let ship_id = ship.id;
            let enemies = 
                &self.context.ships_iter()
                    .filter(|s| s.id != ship_id && !s.exploding)
                    .collect();
            
            if ship.client_id.is_none() {
                // Run AI
                run_ai(ship, enemies);
            }
        }
        
        // Let the ships that want to jump jump, if they can
        for ship in self.context.ships_iter_mut() {
            if ship.target_sector.is_some() {
                ship.jumping = true;
            }
        }
        
        // Apply all the plans
        for (ship, plans) in self.ship_plans.drain() {
            ship.get_mut(&mut self.context).apply_plans(&plans);
        }
    
        // Do server-side precalculations
        self.context.server_preprocess();
        
        // Send the results packet
        let mut results_packet = self.build_results_packet();
        self.slot.broadcast(results_packet);
        
        // Run the simulation
        self.do_simulation();
        
        // Finish the results packet with ships to add and remove
        let mut new_ships = vec!();
        let mut dead_ships = vec!();
        for ship in self.context.ships_iter() {
            // Replace dead ships with better ships
            if ship.exploding {
                let mut better_ship = Ship::generate(ship.id, ship.name.clone(), ship.level + 1);
                better_ship.client_id = ship.client_id;
                
                // Remove the old ship
                dead_ships.push(ship.index);
                
                // Add the better ship
                new_ships.push(better_ship);
            }
        }
        
        for dead_ship in dead_ships.into_iter() {
            self.context.remove_ship(dead_ship);
            self.ships_to_remove.push(dead_ship);
        }
        
        for new_ship in new_ships.into_iter() {
            let ship_index = self.context.add_ship(new_ship);
            self.ships_to_add.push(ship_index);
        }
        
        // Handle all the ships that need to start exploding
        //let mut exploding_ships = vec!();
        for ship in self.context.ships_iter_mut() {            
            // Replace dead ships with better ships
            if ship.state.get_hp() == 0 {
                ship.exploding = true;
                //exploding_ships.push(ship.id);
            }
        }
        //let mut exploding_packet = OutPacket::new();
        //exploding_packet.write(&exploding_ships);
        //self.slot.broadcast(exploding_packet);
        
        // Send off all the ships that jumped
        let mut jumped_ships = vec!();
        for ship in self.context.ships_iter() {
            if let Some(sector_id) = ship.target_sector {
                jumped_ships.push(ship.index);
                self.ships_to_remove.push(ship.index);
            }
        }
        
        // Send new ships
        self.send_new_ships();
        
        for jumped_ship in jumped_ships.into_iter() {
            use std::rc::try_unwrap;
            
            let ship = self.context.remove_ship(jumped_ship);

            if let Some(client_id) = ship.client_id {
                // Send the last tick
                self.send_last_tick(client_id);
            
                let ship_stored = ShipStored::from_ship(ship);
                
                let mut account = self.accounts.remove(&client_id).expect("Client's account must exist here.");
                account.ship = Some(ship_stored);
                
                self.slot.transfer_client(client_id, self.star_map_slot_id);
                
                to_map_sender.send(account);
            }
        }
        
        // Reset everything for the next turn
        self.received_plans.clear();
        self.turn_number += 1;
        
        // Transfer waiting clients to active clients
        self.clients_active = self.clients_active.union(&self.clients_waiting).map(|&x| x).collect();
    }
    
    fn do_simulation(&mut self) {
        let mut sim_events = SimEvents::new();
    
        // Pre simulation
        self.context.before_simulation(&mut sim_events);
        
        // Simulation!!!
        self.simulate(&mut sim_events);
        
        // Post simulation
        self.context.after_simulation();
        
        // Apply module stats
        self.context.apply_module_stats();
        
        // Deactivate modules that can no longer be powered
        self.context.deactivate_unpowerable_modules();
    }
    
    fn simulate(&mut self, sim_events: &mut SimEvents) {
        for tick in 0..100 {
            sim_events.apply_tick(&mut self.context, tick);
        }
    }
    
    fn build_results_packet(&mut self) -> OutPacket {
        let mut packet = OutPacket::new();
        match packet.write(&ClientPacketId::SimResults) {
            Ok(()) => {},
            Err(_) => panic!("Failed to write results packet ID"),
        }
        
        // Write the results!
        self.context.write_results(&mut packet);

        packet
    }
    
    fn send_new_ships(&mut self) {
        if self.debug {
            println!("Sending new ships");
        }
    
        let mut ships_packet = OutPacket::new();
        
        {
            let ships_to_add: Vec<&Ship> = self.ships_to_add.iter().map(|s| s.get(&self.context)).collect();
            ships_packet.write(&ships_to_add);
        }
        
        ships_packet.write(&self.ships_to_remove);
        self.slot.broadcast(ships_packet);
        
        self.ships_to_add.clear();
        self.ships_to_remove.clear();
    }
    
    fn send_turn_tick(&mut self) {
        if self.debug {
            println!("Sending tick");
        }

        let mut packet = OutPacket::new();
        packet.write(&ClientPacketId::Tick);
        self.slot.broadcast(packet);
    }
    
    fn send_last_tick(&mut self, client_id: ClientId) {
        if self.debug {
            println!("Sending tick");
        }

        let mut packet = OutPacket::new();
        packet.write(&ClientPacketId::LastTick);
        self.slot.send(client_id, packet);
    }
}
