use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{Sender, Receiver};
use time;

use login::AccountBox;
use ai::run_ai;
use battle_state::{BattleContext, ClientPacketId, ServerPacketId};
use module::Module;
use net::{ClientId, ServerSlot, SlotInMsg, InPacket, OutPacket};
use ship::{Ship, ShipId, ShipStored};
use sim::SimEvents;

pub struct SectorState {
    slot: ServerSlot,

    // Context holding all the things involved in this battle
    context: BattleContext,
    
    turn_start_time: time::Timespec,
    
    received_plans: HashSet<ClientId>,
    clients_waiting: HashSet<ClientId>,
    clients_active: HashSet<ClientId>,
    
    // All the clients' accounts
    accounts: HashMap<ClientId, AccountBox>,
    
    // Ships to add after simulation
    ships_to_add: Vec<Ship>,
    
    // Ships to remove after simulation
    ships_to_remove: Vec<ShipId>,
    
    turn_number: u32,
}

impl SectorState {
    pub fn new(slot: ServerSlot, context: BattleContext) -> SectorState {
        SectorState {
            slot: slot,
            context: context,
            turn_start_time: time::now().to_timespec(),
            received_plans: HashSet::new(),
            clients_waiting: HashSet::new(),
            clients_active: HashSet::new(),
            accounts: HashMap::new(),
            ships_to_add: vec!(),
            ships_to_remove: vec!(),
            turn_number: 0,
        }
    }
    
    pub fn run(&mut self, to_map_sender: Sender<AccountBox>, from_map_receiver: Receiver<AccountBox>) {
        // TODO: come up with better way to generate AI ship IDs
        let mut ai_ship = Ship::generate((100000000) as ShipId, 2);
        self.context.add_ship(Rc::new(RefCell::new(ai_ship)));
    
        loop {
            if let Ok(msg) = self.slot.try_receive() {
                match msg {
                    SlotInMsg::Joined(client_id) => {
                        println!("Client {} joined battle {}", client_id, self.slot.get_id());
                    },
                    SlotInMsg::ReceivedPacket(client_id, mut packet) => { self.handle_packet(client_id, &mut packet); },
                    _ => {}
                }
            }
            
            if let Ok(mut account) = from_map_receiver.try_recv() {
                let client_id = account.client_id.expect("This must have a client ID");
                
                // Get the ship out of storage
                let ship_stored = account.ship.take().expect("This account must have a ship");
                let ship = ship_stored.to_ship(Some(client_id));
                
                // Get the current time from our turn timer
                let turn_time = time::now().to_timespec() - self.turn_start_time;
            
                // Send the player all the ships
                let mut packet = OutPacket::new();
                
                if self.clients_active.is_empty() {
                    self.clients_active.insert(client_id);
                    packet.write(&0).unwrap();
                } else {
                    self.clients_waiting.insert(client_id);
                    packet.write(&(turn_time.num_milliseconds() as u32)).unwrap();
                }
                
                // Add the player's account
                self.accounts.insert(client_id, account);
                
                // Add the player's ship
                self.context.add_ship(Rc::new(RefCell::new(ship)));
                
                packet.write(&self.context.ships_list).unwrap();
                self.slot.send(client_id, packet);
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
                self.received_plans.insert(client_id);
                
                // Handle the plans
                self.handle_plans_packet(client_id, packet);
 
                if self.received_plans == self.clients_active {
                    // Run AI on ships with no client
                    for ship in self.context.ships_list.iter() {
                        let ship_id = ship.borrow().id;
                        let enemies = &self.context.ships_list.iter().filter(|s| s.borrow().id != ship_id).map(|s| s.clone()).collect();
                        
                        let mut ship = ship.borrow_mut();
                        if ship.client_id.is_none() {
                            // Run AI
                            run_ai(ship.deref_mut(), enemies);
                            ship.apply_module_plans();
                        }
                    }
                
                    // Do server-side precalculations
                    self.context.server_preprocess();
                    
                    // Start building the results packet
                    let mut results_packet = self.build_results_packet();
                    
                    // Run the simulation
                    self.do_simulation();
                    
                    // Finish the results packet with ships to add and remove
                    for ship in self.context.ships_list.iter() {
                        let ship = ship.borrow();
                        
                        // Replace dead ships with better ships
                        if ship.state.get_hp() == 0 {
                            let mut better_ship = Ship::generate(ship.id, ship.level + 1);
                            better_ship.client_id = ship.client_id;
                            
                            self.ships_to_add.push(better_ship);
                            self.ships_to_remove.push(ship.id);
                        }
                    }
                    
                    results_packet.write(&self.ships_to_add);
                    results_packet.write(&self.ships_to_remove);
                    self.slot.broadcast(results_packet);
                    
                    for ship in self.ships_to_remove.drain() {
                        self.context.remove_ship(ship);
                    }
                    
                    for ship in self.ships_to_add.drain() {
                        self.context.add_ship(Rc::new(RefCell::new(ship)));
                    }
                    
                    // Reset everything for the next turn
                    self.received_plans.clear();
                    self.turn_number += 1;
                    
                    // Transfer waiting clients to active clients
                    self.clients_active = self.clients_active.union(&self.clients_waiting).map(|&x| x).collect();
                    
                    // Reset the turn timer
                    self.turn_start_time = time::now().to_timespec();
                }
            },
        }
    }
    
    fn handle_plans_packet(&mut self, client_id: ClientId, packet: &mut InPacket) {
        self.context.get_ship_by_client_id(client_id).borrow_mut().read_plans(&self.context, packet);
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
        for tick in range(0u32, 100) {
            sim_events.apply_tick(tick);
        }
    }
    
    fn build_results_packet(&mut self) -> OutPacket {
        let mut packet = OutPacket::new();
        match packet.write(&ClientPacketId::SimResults) {
            Ok(()) => {},
            Err(_) => panic!("Failed to write results packet ID"),
        }
        
        // The results packet has both the plans and the results, because clients need both
        self.context.write_plans(&mut packet);
        self.context.write_results(&mut packet);

        packet
    }
}
