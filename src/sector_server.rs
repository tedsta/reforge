use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use time;

use rand::Rng;
use rand;

use ai::run_ai;
use battle_context::BattleContext;
use chat::ChatMsg;
use login::AccountBox;
use module::{ModelStore, Module};
use net::{ClientId, ServerSlot, ServerSlotId, SlotInMsg, InPacket, OutPacket};
use packet_types::{ClientBattlePacket, ServerBattlePacket};
use ship::{Ship, ShipId, ShipIndex, ShipPlans, ShipStored};
use sim::SimEvents;
use star_map::StarMapAction;
use vec::Vec2;

pub struct SectorState {
    slot: ServerSlot,
    star_map_slot_id: ServerSlotId,
    chat_sender: Sender<ChatMsg>,
    chat_receiver: Receiver<ChatMsg>,
    to_map_sender: Sender<(AccountBox, StarMapAction)>,
    from_map_receiver: Receiver<AccountBox>,

    // Context holding all the things involved in this battle
    context: BattleContext,
    
    model_store: Arc<ModelStore>,
    
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
    
    ships_to_logout: Vec<ShipIndex>,
    
    turn_number: u32,
    
    debug: bool,
}

impl SectorState {
    pub fn new(slot: ServerSlot,
               star_map_slot_id: ServerSlotId,
               chat_sender: Sender<ChatMsg>,
               chat_receiver: Receiver<ChatMsg>,
               to_map_sender: Sender<(AccountBox, StarMapAction)>,
               from_map_receiver: Receiver<AccountBox>,
               context: BattleContext,
               model_store: Arc<ModelStore>,
               debug: bool) -> SectorState {
        SectorState {
            slot: slot,
            star_map_slot_id: star_map_slot_id,
            chat_sender: chat_sender,
            chat_receiver: chat_receiver,
            to_map_sender: to_map_sender,
            from_map_receiver: from_map_receiver,
            context: context,
            model_store: model_store,
            turn_start_time: time::now().to_timespec(),
            simulated_turn: false,
            received_plans: HashSet::new(),
            clients_waiting: HashSet::new(),
            clients_active: HashSet::new(),
            ship_plans: vec!(),
            accounts: HashMap::new(),
            ships_to_add: vec!(),
            ships_to_remove: vec!(),
            ships_to_logout: vec!(),
            turn_number: 0,
            debug: debug,
        }
    }
    
    pub fn run(&mut self, ack: Sender<()>) {
        let mut rng = rand::thread_rng();
    
        loop {
            ///////////////////////////////////////////////////////////
            // Check if it's time to simulate next turn
            
            // Get the current time from our turn timer
            let turn_time = time::now().to_timespec() - self.turn_start_time;
            
            if !self.simulated_turn && turn_time.num_milliseconds() >= 3500 {
                self.simulate_next_turn();

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
                    SlotInMsg::Disconnected(client_id) => {
                        println!("Client {} disconnected at station {}, logging out...", client_id, self.slot.get_id());
                        
                        let ship = self.context.get_ship_by_client_id(client_id);
                        self.ships_to_logout.push(ship.index);
                    },
                    SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                        self.handle_packet(client_id, &mut packet);
                    },
                }
            }
            
            ///////////////////////////////////////////////////////////
            // Receive messages from chat server
            if let Ok(msg) = self.chat_receiver.try_recv() {
                let mut msg_packet = OutPacket::new();
                msg_packet.write(&ClientBattlePacket::Chat(msg)).unwrap();
                self.slot.broadcast(msg_packet);
            }
            
            ///////////////////////////////////////////////////////////
            // Receive new clients
            if let Ok(mut account) = self.from_map_receiver.try_recv() {
                if self.debug {
                    println!("Receiving account");
                }
                println!("Receiving account {}", self.simulated_turn);
                let client_id = account.client_id.expect("This must have a client ID");
                
                // Add the client to the waiting list
                self.clients_waiting.insert(client_id);
                
                // Get the ship out of storage
                let ship_stored = account.ship.take().expect("This account must have a ship");
                let mut ship = ship_stored.to_ship(Some(client_id));
                
                ship.position = Vec2::new(rng.gen::<f64>() * 300.0 - 150.0, rng.gen::<f64>() * 300.0 - 150.0);
                
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
        let battle_packet: ServerBattlePacket = packet.read().unwrap();
        
        match battle_packet {
            ServerBattlePacket::Plan => { self.handle_plans(client_id, packet); },
            ServerBattlePacket::Chat(msg) => {
                if self.debug {
                    println!("Handling chat packet");
                }
                
                let ref account = self.accounts[&client_id];
            
                let msg = ChatMsg {
                    author_name: account.username.clone(),
                    content: msg,
                };
                
                self.chat_sender.send(msg);
            },
            ServerBattlePacket::Logout => {
                let ship = self.context.get_ship_by_client_id(client_id);
                self.ships_to_logout.push(ship.index);
            },
        }
    }
    
    fn handle_plans(&mut self, client_id: ClientId, packet: &mut InPacket) {
        if self.debug {
            println!("Handling plans packet");
        }
    
        self.received_plans.insert(client_id);
    
        let ship = self.context.get_ship_by_client_id(client_id);
        
        let plans = packet.read().unwrap();
        
        if !ship.exploding {
            // Don't save these plans if the ship is exploding
            self.ship_plans.push((ship.index, plans));
        }
        
        println!("Received plans packet from {} for turn {}", client_id, self.turn_number);
 
        if self.received_plans == self.clients_active {
            // TODO decide if we should simulate turn a little early here.
        }
    }
    
    fn simulate_next_turn(&mut self) {
        if self.debug {
            println!("Simulating next turn");
        }
    
        // Send new ships to added/removed before simulation
        self.send_new_ships_pre();
    
        // Run AI on ships with no client
        for ship in self.context.ships_iter() {
            let ship_id = ship.id;
            let enemies = 
                &self.context.ships_iter()
                    .filter(|s| s.id != ship_id && !s.exploding)
                    .collect();
            
            if ship.client_id.is_none() {
                // Run AI
                let mut plans = ship.create_plans();
                run_ai(ship, &mut plans, enemies);
                self.ship_plans.push((ship.index, plans));
            }
        }
        
        // Apply all the plans
        let mut jumped_ships = vec!();
        for (ship, plans) in self.ship_plans.drain(..) {
            let ship = ship.get_mut(&mut self.context);
        
            ship.apply_plans(&plans);
            
            // Handle jumping ships
            if let Some(target_sector) = plans.target_sector {
                ship.jumping = true;
                
                jumped_ships.push((ship.index, target_sector));
                self.ships_to_remove.push(ship.index);
            }
        }
        
        // Handle ships that are logging out
        for logging_out_ship in &self.ships_to_logout {
            self.ships_to_remove.push(*logging_out_ship);
        }
    
        // Do server-side precalculations
        self.context.server_preprocess(&*self.model_store);
        
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
                let next_level = cmp::min(ship.level + 1, 15);
                let mut better_ship = Ship::generate(ship.id, ship.name.clone(), next_level);
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
        
        // Make dead ships start exploding
        for ship in self.context.ships_iter_mut() {
            if ship.state.get_hp() == 0 {
                ship.exploding = true;
            }
        }
        
        // Send new ships
        self.send_new_ships_post();
        
        // Send off all of the jumping ships
        for (jumped_ship, target_sector) in jumped_ships.into_iter() {
            let ship = self.context.remove_ship(jumped_ship);

            if let Some(client_id) = ship.client_id {
                // Send the last tick
                self.send_final_ticks(client_id, 1);
                
                let ship_stored = ShipStored::from_ship(ship);
                
                let mut account = self.accounts.remove(&client_id).expect("Client's account must exist here.");
                account.ship = Some(ship_stored);
                
                self.slot.transfer_client(client_id, self.star_map_slot_id);
                
                self.to_map_sender.send((account, StarMapAction::Jump(target_sector)));
            }
        }
        
        // Send off all of the ships logging out
        for ship in &self.ships_to_logout {
            let ship = self.context.remove_ship(*ship);

            if let Some(client_id) = ship.client_id {
                // Send the last tick
                self.send_final_ticks(client_id, 1);
                
                let ship_stored = ShipStored::from_ship(ship);
                
                let mut account = self.accounts.remove(&client_id).expect("Client's account must exist here.");
                account.ship = Some(ship_stored);
                
                self.slot.transfer_client(client_id, self.star_map_slot_id);
                
                self.to_map_sender.send((account, StarMapAction::Logout));
            }
        }
        self.ships_to_logout.clear();
        
        // Reset everything for the next turn
        self.received_plans.clear();
        self.turn_number += 1;
        
        // Transfer waiting clients to active clients
        self.clients_active = self.clients_active.union(&self.clients_waiting).map(|&x| x).collect();
    }
    
    fn do_simulation(&mut self) {
        let mut sim_events = SimEvents::new();
    
        // Pre simulation
        self.context.before_simulation(&*self.model_store, &mut sim_events);
        
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
        packet.write(&ClientBattlePacket::SimResults).unwrap();
        self.context.write_results(&mut packet);
        packet
    }
    
    fn send_new_ships_pre(&mut self) {
        let mut ships_packet = OutPacket::new();
        ships_packet.write(&ClientBattlePacket::NewShipsPre).unwrap();
        self.write_new_ships(&mut ships_packet);
        self.slot.broadcast(ships_packet);
    }
    
    fn send_new_ships_post(&mut self) {
        let mut ships_packet = OutPacket::new();
        ships_packet.write(&ClientBattlePacket::NewShipsPost).unwrap();
        self.write_new_ships(&mut ships_packet);
        self.slot.broadcast(ships_packet);
    }
    
    fn write_new_ships(&mut self, ships_packet: &mut OutPacket) {
        if self.debug {
            println!("Sending new ships");
        }
        
        {
            let ships_to_add: Vec<&Ship> = self.ships_to_add.iter().map(|s| s.get(&self.context)).collect();
            ships_packet.write(&ships_to_add);
        }
        
        ships_packet.write(&self.ships_to_remove);
        
        self.ships_to_add.clear();
        self.ships_to_remove.clear();
    }
    
    fn send_turn_tick(&mut self) {
        if self.debug {
            println!("Sending tick");
        }

        let mut packet = OutPacket::new();
        packet.write(&ClientBattlePacket::Tick(None));
        self.slot.broadcast(packet);
    }
    
    fn send_final_ticks(&self, client_id: ClientId, ticks_left: u8) {
        if self.debug {
            println!("Sending tick");
        }

        let mut packet = OutPacket::new();
        packet.write(&ClientBattlePacket::Tick(Some(ticks_left)));
        self.slot.send(client_id, packet);
    }
}
