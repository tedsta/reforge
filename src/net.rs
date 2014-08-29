use std::collections::HashMap;
use std::io::{TcpListener, TcpStream, Acceptor, Listener};
use std::io::{MemReader, MemWriter, IoResult, TimedOut};

///////////////////////////////////////////////////////////////////////////////////////////////////
// Some basic types

pub type ClientId = u32;

pub type ServerSlotId = u32;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server Slot

// Messages incoming to slots
pub enum SlotInMsg {
    Joined(ClientId),                        // Client joined slot (client_id)
    Disconnected(ClientId),                  // Client was disconnected from server (client_id)
    ReceivedPacket(ClientId, Box<InPacket>), // Received packet from client (client_id, packet)
}

// Messages outgoing from slots
pub enum SlotOutMsg {
    SendPacket(ClientId, OutPacket), // Received packet from client (client_id, packet)
    CreateSlot(ServerSlotId),            // Tell the server to make a new ServerSlot (slot_id)
    TransferClient(ClientId, ServerSlotId), // Tell the server to transfer a client to a different slot
}

pub struct ServerSlot {
    id: ServerSlotId,
    sender: Sender<SlotOutMsg>,
    receiver: Receiver<SlotInMsg>,
    
    // When this server slot requests to make a new slot, the new slot will come on this channel.
    create_slot: Receiver<Box<ServerSlot>>,
}

impl ServerSlot {
    fn new(id: ServerSlotId, sender: Sender<SlotOutMsg>, receiver: Receiver<SlotInMsg>, create_slot: Receiver<Box<ServerSlot>>) -> ServerSlot {
        ServerSlot{id: id, sender: sender, receiver: receiver, create_slot: create_slot}
    }
    
    pub fn send(&self, client_id: ClientId, packet: OutPacket) {
        self.sender.send(SendPacket(client_id, packet));
    }
    
    pub fn receive(&self) -> SlotInMsg {
        self.receiver.recv()
    }
    
    pub fn create_slot(&self) -> Box<ServerSlot> {
        self.sender.send(CreateSlot(self.id));
        self.create_slot.recv()
    }
    
    // Transfer a client to a different lost
    pub fn transfer_client(&self, client_id: ClientId, to_slot: ServerSlotId) {
        self.sender.send(TransferClient(client_id, to_slot));
    }
    
    pub fn id(&self) -> ServerSlotId {
        self.id
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server

pub struct Server {
    // Server slots. Maps slot ID to communication channels with slot
    slots: HashMap<ServerSlotId, (Sender<SlotInMsg>, Sender<Box<ServerSlot>>)>,
    
    // Channel for communication between server master task and slots
    slot_channel_t: Sender<SlotOutMsg>,
    slot_channel_r: Receiver<SlotOutMsg>,
    
    // ID to give to next slot
    next_slot_id: ServerSlotId,
}

impl Server {
    pub fn new() -> Server {
        let (slot_channel_t, slot_channel_r) = channel();
    
        Server {
            slots: HashMap::new(),
            slot_channel_t: slot_channel_t, slot_channel_r: slot_channel_r,
            next_slot_id: 0,
        }
    }
    
    pub fn create_slot(&mut self) -> ServerSlot {
        let (slot_in_t, slot_in_r) = channel();
        let (create_slot_t, create_slot_r) = channel(); // Channel for sending newly created ServerSlots to the slot upon request
        
        let slot_id = self.next_slot_id;
        self.slots.insert(slot_id, (slot_in_t, create_slot_t));
        self.next_slot_id += 1;
    
        ServerSlot::new(slot_id, self.slot_channel_t.clone(), slot_in_r, create_slot_r)
    }
    
    pub fn listen(&mut self, port: u16) {
        let listener = TcpListener::bind("0.0.0.0", port).ok().unwrap();
        
        let mut acceptor = listener.listen().ok().unwrap();
        acceptor.set_timeout(Some(0));
        
        // Maps clients to their server slots
        let mut client_slots: HashMap<ClientId, Sender<SlotInMsg>> = HashMap::new();
        
        // Maps clients to their out packet channels
        let mut client_outs: HashMap<ClientId, Sender<OutPacket>> = HashMap::new();
        
        // Client task to master: packet channel
        let (packet_in_t, packet_in_r): (Sender<(ClientId, Box<InPacket>)>, Receiver<(ClientId, Box<InPacket>)>) = channel();
        
        // Next ID to give to each client
        let mut next_client_id = 0;
        
        // Manage server slots
        loop {
            // Accept connections and process them, spawning a new tasks for each one
            let mut accepted_connections = 0u; // Counter for accepted connections - move on after a while if connections keep coming
            for stream in acceptor.incoming() {
                match stream {
                    Err(ref e) if e.kind == TimedOut => { break }, // TimedOut is fine, because timeout is 0 lolz
                    Err(e) => println!("Incoming connection failed: {}", e),
                    Ok(stream) => {
                        let client_id = next_client_id;
                        next_client_id += 1;
                        
                        // Assign client to default slot
                        let (ref default_slot, _) = self.slots[0];
                        client_slots.insert(client_id, default_slot.clone());
                        
                        // Create client packet output channel
                        let (client_out_t, client_out_r) = channel();
                        client_outs.insert(client_id, client_out_t);
                        
                        // Clone packet in channel
                        let packet_in_t = packet_in_t.clone();
                        
                        // Clone stream for output stream
                        let out_stream = stream.clone(); // Create copy of stream for output
                    
                        // Client input process
                        spawn(proc() {
                            handle_client_in(client_id, stream, packet_in_t);
                        });
                        
                        // Client output process
                        spawn(proc() {
                            handle_client_out(out_stream, client_out_r);
                        });
                        
                        // Tell the default channel that it's been joined
                        default_slot.send(Joined(client_id));
                        
                        accepted_connections += 1;
                    }
                }
                
                if accepted_connections >= 5 {
                    break;
                }
            }
        
            // Check for new packets
            let mut received_packets = 0u; // Packet counter. Move on after a while if packets keep coming
            loop {
                match packet_in_r.try_recv() {
                    Ok((client_id, packet)) => {
                        received_packets += 1;
                        
                        // Send the received packet to the slot the client is in
                        client_slots[client_id].send(ReceivedPacket(client_id, packet));
                    },
                    Err(_) => break
                }
                
                if received_packets >= 10 {
                    break;
                }
            }
            
            // Check for messages from slots
            let mut received_messages = 0u; // Packet counter. Move on after a while if messages keep coming
            loop {
                match self.slot_channel_r.try_recv() {
                    Ok(msg) => {
                        received_messages += 1;
                        match msg {
                            SendPacket(client_id, packet) => match client_outs.find(&client_id) {
                                Some(c) => c.send(packet),
                                None => println!("Failed to send packet to invalid client ID {}", client_id)
                            },
                            CreateSlot(slot_id) =>  {
                                let new_slot = box self.create_slot();
                                let (_, ref create_slot_t) = self.slots[slot_id];
                                create_slot_t.send(new_slot);
                            },
                            TransferClient(client_id, slot_id) => {
                                match self.slots.find(&slot_id) {
                                    Some(slot) => {
                                        let &(ref slot_in_t, _) = slot;
                                        client_slots.get_mut(&client_id).clone_from(slot_in_t);
                                        slot_in_t.send(Joined(client_id));
                                    },
                                    None => fail!("Failed to transfer client {} to non-existant slot {}", client_id, slot_id)
                                }
                            },
                        }
                    },
                    Err(_) => break
                }
                
                if received_messages >= 10 {
                    break;
                }
            }
        }
    }
}

fn handle_client_in(client_id: ClientId, mut stream: TcpStream, packet_in_t: Sender<(ClientId, Box<InPacket>)>) {
    loop {
        packet_in_t.send((client_id, box InPacket::new_from_reader(&mut stream)));
    }
}

fn handle_client_out(mut stream: TcpStream, out_r: Receiver<OutPacket>) {
    loop {
        // Receive a packet to send
        let packet = out_r.recv();
        
        // Get the packet's data
        let data = packet.writer.get_ref();
        
        // Write the packet size, then the actual packet data
        stream.write_le_u16(data.len() as u16).unwrap();
        stream.write(data).unwrap();
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Client

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(host: &str, port: u16) -> Client {
        Client{stream: TcpStream::connect(host, port).unwrap()}
    }
    
    pub fn send(&mut self, packet: &OutPacket) {
        let data = packet.writer.get_ref();
        self.stream.write_le_u16(data.len() as u16).unwrap();
        self.stream.write(data).unwrap();
    }
    
    pub fn receive(&mut self) -> InPacket {
        InPacket::new_from_reader(&mut self.stream)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Packet

pub struct OutPacket {
    writer: MemWriter,
}

impl OutPacket {
    pub fn new() -> OutPacket {
        OutPacket{writer: MemWriter::new()}
    }
    
    pub fn len(&self) -> uint {
        self.writer.get_ref().len()
    }
    
    pub fn write_i32(&mut self, data: i32) -> IoResult<()> {
        self.writer.write_le_i32(data)
    }
    
    pub fn write_u32(&mut self, data: u32) -> IoResult<()> {
        self.writer.write_le_u32(data)
    }
    
    pub fn write_i16(&mut self, data: i16) -> IoResult<()> {
        self.writer.write_le_i16(data)
    }
    
    pub fn write_u16(&mut self, data: u16) -> IoResult<()> {
        self.writer.write_le_u16(data)
    }
}

pub struct InPacket {
    reader: MemReader,
}

impl InPacket {
    pub fn new(data: Vec<u8>) -> InPacket {
        InPacket{reader: MemReader::new(data)}
    }
    
    pub fn new_from_reader<T: Reader>(reader: &mut T) -> InPacket {
        // Get next packet size
        let packet_size = match reader.read_le_u16() {
                Err(e) => fail!("Failed to receive packet size: {}", e),
                Ok(packet_size) => packet_size
            };
        
        // Get data
        let data = match reader.read_exact(packet_size as uint) {
                Err(e) => fail!("Failed to receive data: {}", e),
                Ok(data) => data
            };
        
        // Build packet
        InPacket::new(data)
    }
    
    pub fn len(&self) -> uint {
        self.reader.get_ref().len()
    }
    
    pub fn read_i32(&mut self) -> IoResult<i32> {
        self.reader.read_le_i32()
    }
    
    pub fn read_u32(&mut self) -> IoResult<u32> {
        self.reader.read_le_u32()
    }
    
    pub fn read_i16(&mut self) -> IoResult<i16> {
        self.reader.read_le_i16()
    }
    
    pub fn read_u16(&mut self) -> IoResult<u16> {
        self.reader.read_le_u16()
    }
}