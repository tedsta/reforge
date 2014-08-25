use std::io::{TcpListener, TcpStream, Acceptor, Listener};
use std::io::{MemReader, MemWriter, IoResult, TimedOut};

use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server Slot

// Messages incoming to slots
pub enum SlotInMsg {
    Joined(u32),                      // Client joined slot (client_id)
    Disconnected(u32),                // Client was disconnected from server (client_id)
    ReceivedPacket(u32, InPacket), // Received packet from client (client_id, packet)
}

// Messages outgoing from slots
pub enum SlotOutMsg {
    SendPacket(u32, OutPacket), // Received packet from client (client_id, packet)
}

pub struct ServerSlot {
    sender: Sender<SlotOutMsg>,
    receiver: Receiver<SlotInMsg>,
}

impl ServerSlot {
    fn new(sender: Sender<SlotOutMsg>, receiver: Receiver<SlotInMsg>) -> ServerSlot {
        ServerSlot{sender: sender, receiver: receiver}
    }
    
    pub fn send(&self, client_id: u32, packet: OutPacket) {
        self.sender.send(SendPacket(client_id, packet));
    }
    
    pub fn receive(&self) -> SlotInMsg {
        self.receiver.recv()
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server

pub struct Server {
    // Server slots. Maps slot ID to communication channels with slot
    slots: HashMap<u32, Sender<SlotInMsg>>,
    
    // Channel for communication between server master task and slots
    slot_channel_t: Sender<SlotOutMsg>,
    slot_channel_r: Receiver<SlotOutMsg>,
    
    // ID to give to next slot
    next_slot_id: u32,
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
        
        self.slots.insert(self.next_slot_id, slot_in_t);
        self.next_slot_id += 1;
    
        ServerSlot::new(self.slot_channel_t.clone(), slot_in_r)
    }
    
    pub fn listen(&mut self, port: u16) {
        let listener = TcpListener::bind("127.0.0.1", port).ok().unwrap();
        
        let mut acceptor = listener.listen().ok().unwrap();
        acceptor.set_timeout(Some(0));
        
        // Maps clients to their server slots
        let mut client_slots: HashMap<u32, Sender<SlotInMsg>> = HashMap::new();
        
        // Maps clients to their out packet channels
        let mut client_outs: HashMap<u32, Sender<OutPacket>> = HashMap::new();
        
        // Client task to master: packet channel
        let (packet_in_t, packet_in_r): (Sender<(u32, InPacket)>, Receiver<(u32, InPacket)>) = channel();
        
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
                        client_slots.insert(client_id, self.slots.find(&0).unwrap().clone());
                        
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
                        match client_slots.find(&client_id) {
                            Some(c) => c.send(ReceivedPacket(client_id, packet)),
                            None => println!("Received packet with invalid client ID {}", client_id)
                        }
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
                            }
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

fn handle_client_in(client_id: u32, mut stream: TcpStream, packet_in_t: Sender<(u32, InPacket)>) {
    loop {
        packet_in_t.send((client_id, InPacket::new_from_reader(&mut stream)));
    }
}

fn handle_client_out(mut stream: TcpStream, out_r: Receiver<OutPacket>) {
    loop {
        let mut packet = out_r.recv();
        let mut data = packet.writer.get_ref();
        stream.write_le_u16(data.len() as u16);
        stream.write(data);
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
        self.stream.write_le_u16(data.len() as u16);
        self.stream.write(data);
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
    
    pub fn write_int(&mut self, data: int) -> IoResult<()> {
        self.writer.write_le_int(data)
    }
    
    pub fn write_uint(&mut self, data: uint) -> IoResult<()> {
        self.writer.write_le_uint(data)
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
    
    pub fn read_int(&mut self) -> IoResult<int> {
        self.reader.read_le_int()
    }
    
    pub fn read_uint(&mut self) -> IoResult<uint> {
        self.reader.read_le_uint()
    }
}