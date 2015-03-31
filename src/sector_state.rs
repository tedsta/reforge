use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{Sender, Receiver};
use time;

use ai::run_ai;
use battle_state::{BattleContext, ClientPacketId, ServerPacketId};
use login::AccountBox;
use module::Module;
use net::{ClientId, ServerSlot, ServerSlotId, SlotInMsg, InPacket, OutPacket};
use ship::{Ship, ShipId, ShipRef, ShipStored, ShipNetworked, as_networked_ships};
use sim::SimEvents;

pub struct SectorState {
    slot: ServerSlot,
    star_map_slot_id: ServerSlotId,

    // Context holding all the things involved in this battle
    context: BattleContext,
    
    turn_start_time: time::Timespec,
    sent_new_ships: bool,
    
    received_plans: HashSet<ClientId>,
    clients_waiting: HashSet<ClientId>,
    clients_active: HashSet<ClientId>,
    
    // All the clients' accounts
    accounts: HashMap<ClientId, AccountBox>,
    
    // Ships to add after simulation
    ships_to_add: Vec<ShipRef>,
    
    // Ships to remove after simulation
    ships_to_remove: Vec<ShipId>,
    
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
            sent_new_ships: false,
            received_plans: HashSet::new(),
            clients_waiting: HashSet::new(),
            clients_active: HashSet::new(),
            accounts: HashMap::new(),
            ships_to_add: vec!(),
            ships_to_remove: vec!(),
            turn_number: 0,
            debug: debug,
        }
    }
    
    pub fn run(&mut self, to_map_sender: Sender<AccountBox>, from_map_receiver: Receiver<AccountBox>, ack: Sender<()>) {
        // TODO: come up with better way to generate AI ship IDs
        let ai_ship = Ship::generate((100000000) as ShipId, "n00bslayer808".to_string(), 2);
        self.context.add_ship(Rc::new(RefCell::new(ai_ship)));
        
        let ai_ship = Ship::generate((100000001) as ShipId, "thing1".to_string(), 2);
        self.context.add_ship(Rc::new(RefCell::new(ai_ship)));
        
        let ai_ship = Ship::generate((100000002) as ShipId, "thing2".to_string(), 2);
        self.context.add_ship(Rc::new(RefCell::new(ai_ship)));
        
        let ai_ship = Ship::generate((100000003) as ShipId, "daisy_girl".to_string(), 2);
        self.context.add_ship(Rc::new(RefCell::new(ai_ship)));
    
        loop {
            ///////////////////////////////////////////////////////////
            // Check if it's time to simulate next turn
            
            // Get the current time from our turn timer
            let turn_time = time::now().to_timespec() - self.turn_start_time;
            
            if !self.sent_new_ships && turn_time.num_milliseconds() > 4500 {
                self.send_new_ships();
                
                self.sent_new_ships = true;
            }
            
            if turn_time.num_milliseconds() > 11000 {
                self.simulate_next_turn(&to_map_sender);
                
                // Reset the turn stuff
                self.turn_start_time = time::now().to_timespec();
                self.sent_new_ships = false;
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
                packet.write(&ShipNetworked::from_ship(&ship));
                packet.write(&self.sent_new_ships); // Whether or not to start at simulation instead of planning phase
                packet.write(&as_networked_ships(&self.context.ships_list)).unwrap();
                self.slot.send(client_id, packet);
                
                // Add the player's ship
                let ship = Rc::new(RefCell::new(ship));
                self.context.add_ship(ship.clone());
                self.ships_to_add.push(ship);
                
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
        let mut ship = self.context.get_ship_by_client_id(client_id).borrow_mut();
    
        ship.target_sector = packet.read().ok().expect("Failed to read player's target sector");
        let plans = packet.read().ok().expect("Failed to read player's plans");
        ship.set_module_plans(&self.context, &plans);
    }
    
    fn simulate_next_turn(&mut self, to_map_sender: &Sender<AccountBox>) {
        if self.debug {
            println!("Simulating next turn");
        }
    
        // Send new ships to added/removed before simulation
        self.send_new_ships();
    
        // Run AI on ships with no client
        for ship in self.context.ships_list.iter() {
            let ship_id = ship.borrow().id;
            let enemies = &self.context.ships_list.iter().filter(|s| s.borrow().id != ship_id).map(|s| s.clone()).collect();
            
            let mut ship = ship.borrow_mut();
            if ship.client_id.is_none() {
                // Run AI
                run_ai(ship.deref_mut(), enemies);
            }
        }
        
        // Let the ships that want to jump jump, if they can
        for ship in self.context.ships_list.iter() {
            let mut ship = ship.borrow_mut();
            if ship.target_sector.is_some() {
                ship.jumping = true;
            }
        }
        
        // Apply all the plans
        self.context.apply_module_plans();
    
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
        for ship in self.context.ships_list.iter() {
            let ship = ship.borrow();
            
            // Replace dead ships with better ships
            if ship.state.get_hp() == 0 {
                let mut better_ship = Ship::generate(ship.id, ship.name.clone(), ship.level + 1);
                better_ship.client_id = ship.client_id;
                let better_ship = Rc::new(RefCell::new(better_ship));
                
                self.ships_to_add.push(better_ship.clone());
                self.ships_to_remove.push(ship.id);
                
                // Remove the old ship
                dead_ships.push(ship.id);
                
                // Add the better ship
                new_ships.push(better_ship);
            }
        }
        
        for dead_ship in dead_ships.into_iter() {
            self.context.remove_ship(dead_ship);
        }
        
        for new_ship in new_ships.into_iter() {
            self.context.add_ship(new_ship);
        }
        
        // Send off all the ships that jumped
        let mut jumped_ships = vec!();
        for ship in self.context.ships_list.iter() {
            if let Some(sector_id) = ship.borrow().target_sector {
                jumped_ships.push(ship.clone());
                self.ships_to_remove.push(ship.borrow().id);
            }
        }
        
        // Send new ships
        self.send_new_ships();
        
        for jumped_ship in jumped_ships.into_iter() {
            use std::rc::try_unwrap;
            
            let id = jumped_ship.borrow().id;
            self.context.remove_ship(id);
            
            let ship_ref_cell = try_unwrap(jumped_ship).ok().expect("Failed to unwrap jumping ship");
            let mut ship = ship_ref_cell.into_inner();
            
            // The plan power needs to be correct when going to the new sector, and any plans the
            // player made during the last simulation phase are cancelled, so this is safe to do.
            ship.state.plan_power = ship.state.power;

            if let Some(client_id) = ship.client_id {
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
    }
    
    fn simulate(&mut self, sim_events: &mut SimEvents) {
        for tick in 0..100 {
            sim_events.apply_tick(tick);
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
        ships_packet.write(&as_networked_ships(&self.ships_to_add));
        ships_packet.write(&self.ships_to_remove);
        self.slot.broadcast(ships_packet);
        
        self.ships_to_add.clear();
        self.ships_to_remove.clear();
    }
}
