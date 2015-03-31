use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::result::Result;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::{Builder, Thread};

use rustc_serialize::Encodable;
use rustc_serialize::Decodable;

use bincode::{EncoderWriter, EncodingError, DecoderReader, DecodingError, encode_into, decode_from, SizeLimit};

///////////////////////////////////////////////////////////////////////////////////////////////////
// Some basic types

pub type ClientId = u32;

pub type ServerSlotId = u32;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server Slot

// Messages incoming to slots
pub enum SlotInMsg {
    Joined(ClientId),                   // Client joined slot (client_id)
    Disconnected(ClientId),             // Client was disconnected from server (client_id)
    ReceivedPacket(ClientId, InPacket), // Received packet from client (client_id, packet)
}

// Messages outgoing from slots
pub enum SlotOutMsg {
    SendPacket(ServerSlotId, ClientId, OutPacket),        // Send a packet to a client (my_slot_id, client_id, packet)
    BroadcastPacket(ServerSlotId, OutPacket),             // Send packet to all clients in slot (my_slot_id, packet)
    CreateSlot(ServerSlotId),                             // Tell the server to make a new ServerSlot (slot_id)
    TransferClient(ServerSlotId, ClientId, ServerSlotId), // Tell the server to transfer a client to a different slot
}

pub struct ServerSlot {
    id: ServerSlotId,
    sender: Sender<SlotOutMsg>,
    receiver: Receiver<SlotInMsg>,
    
    // When this server slot requests to make a new slot, the new slot will come on this channel.
    create_slot: Receiver<ServerSlot>,
}

impl ServerSlot {
    fn new(id: ServerSlotId, sender: Sender<SlotOutMsg>, receiver: Receiver<SlotInMsg>, create_slot: Receiver<ServerSlot>) -> ServerSlot {
        ServerSlot{id: id, sender: sender, receiver: receiver, create_slot: create_slot}
    }
    
    pub fn send(&self, client_id: ClientId, packet: OutPacket) {
        self.sender.send(SlotOutMsg::SendPacket(self.id, client_id, packet));
    }
    
    pub fn broadcast(&self, packet: OutPacket) {
        self.sender.send(SlotOutMsg::BroadcastPacket(self.id, packet));
    }
    
    pub fn receive(&self) -> SlotInMsg {
        match self.receiver.recv() {
            Ok(msg) => msg,
            _ => panic!("Failed to receive SlotInMsg"),
        }
    }
    
    pub fn try_receive(&self) -> Result<SlotInMsg, TryRecvError> {
        self.receiver.try_recv()
    }
    
    pub fn create_slot(&self) -> ServerSlot {
        self.sender.send(SlotOutMsg::CreateSlot(self.id));
        match self.create_slot.recv() {
            Ok(slot) => slot,
            _ => panic!("Failed to receive newly created ServerSlot"),
        }
    }
    
    // Transfer a client to a different slot
    pub fn transfer_client(&self, client_id: ClientId, to_slot: ServerSlotId) {
        self.sender.send(SlotOutMsg::TransferClient(self.id, client_id, to_slot));
    }
    
    pub fn create_slot_and_transfer_clients(&self, clients: &Vec<ClientId>) -> ServerSlot {
        let new_slot = self.create_slot();
        
        for client_id in clients.iter() {
            self.transfer_client(*client_id, new_slot.get_id());
        }
        
        new_slot
    }
    
    pub fn get_id(&self) -> ServerSlotId {
        self.id
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server

pub struct Server {
    // Server slots. Maps slot ID to communication channels with slot
    slots: HashMap<ServerSlotId, (Sender<SlotInMsg>, Sender<ServerSlot>)>,
    
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
    
    pub fn listen(&mut self, address: &str) {
        let listener = 
            match TcpListener::bind(address) {
                Ok(listener) => listener,
                Err(e) => panic!("Server failed to listen on address {}: {}", address, e),
            };
        
        // Maps clients to their server slots
        let mut client_slots: HashMap<ClientId, Sender<SlotInMsg>> = HashMap::new();
        
        // Maps clients to their out packet channels. Also store client's server slot ID so we can verify it when
        // sending packets from server slots.
        let mut client_outs: HashMap<ClientId, (ServerSlotId, Sender<OutPacket>)> = HashMap::new();
        
        // Client task to master: packet channel
        let (packet_in_t, packet_in_r): (Sender<(ClientId, InPacket)>, Receiver<(ClientId, InPacket)>) = channel();
        
        // Server listener task to master: TcpStream channel
        let (new_client_t, new_client_r): (Sender<TcpStream>, Receiver<TcpStream>) = channel();
        
        // Next ID to give to each client
        let mut next_client_id = 0;
        
        Thread::spawn(move || {
            client_acceptor(listener, new_client_t);
        });
        
        // Manage server slots
        loop {
            // Accept connections and process them, spawning a new tasks for each one
            let mut accepted_connections = 0u32; // Counter for accepted connections - move on after a while if connections keep coming
            loop {
                match new_client_r.try_recv() {
                    Err(_) => { break; },
                    Ok(mut stream) => {
                        let client_id = next_client_id;
                        next_client_id += 1;
                        
                        // Send back the client ID
                        if let Err(e) = write_u32(&mut stream, client_id) {
                            panic!("Failed to send client ID to client: {}", e);
                        }
                        
                        // Assign client to default slot
                        let (ref default_slot, _) = self.slots[&0];
                        client_slots.insert(client_id, default_slot.clone());
                        
                        // Create client packet output channel
                        let (client_out_t, client_out_r) = channel();
                        client_outs.insert(client_id, (0, client_out_t)); // Zero is the slot ID of the default slot
                        
                        // Clone packet in channel
                        let packet_in_t = packet_in_t.clone();
                        
                        // Clone stream for output stream
                        let out_stream = stream.try_clone().ok().expect("Failed to clone to-client stream");
                    
                        // Client input process
                        Thread::spawn(move || {
                            handle_client_in(client_id, stream, packet_in_t);
                        });
                        
                        // Client output process
                        Thread::spawn(move || {
                            handle_client_out(out_stream, client_out_r);
                        });
                        
                        // Tell the default channel that it's been joined
                        default_slot.send(SlotInMsg::Joined(client_id));
                        
                        accepted_connections += 1;
                    }
                }
                
                if accepted_connections >= 5 {
                    break;
                }
            }
        
            // Check for new packets
            let mut received_packets = 0u32; // Packet counter. Move on after a while if packets keep coming
            loop {
                match packet_in_r.try_recv() {
                    Ok((client_id, packet)) => {
                        received_packets += 1;
                        
                        // Send the received packet to the slot the client is in
                        client_slots[&client_id].send(SlotInMsg::ReceivedPacket(client_id, packet));
                    },
                    Err(_) => { break; }
                }
                
                if received_packets >= 10 {
                    break;
                }
            }
            
            // Check for messages from slots
            let mut received_messages = 0u32; // Packet counter. Move on after a while if messages keep coming
            loop {
                match self.slot_channel_r.try_recv() {
                    Ok(msg) => {
                        received_messages += 1;
                        match msg {
                            SlotOutMsg::SendPacket(slot_id, client_id, packet) => match client_outs.get(&client_id) {
                                Some(&(ref client_slot_id, ref c)) => {
                                    if slot_id == *client_slot_id {
                                        c.send(packet);
                                    } else {
                                        println!("Failed to send packet to client {} from server slot {} because the client's server slot is {}", client_id, slot_id, client_slot_id);
                                    }
                                },
                                None => { println!("Failed to send packet to invalid client ID {}", client_id); }
                            },
                            SlotOutMsg::BroadcastPacket(slot_id, packet) => for &(ref client_slot_id, ref c) in client_outs.values() {
                                if slot_id == *client_slot_id {
                                    c.send(packet.clone());
                                }
                            },
                            SlotOutMsg::CreateSlot(slot_id) =>  {
                                let new_slot = self.create_slot();
                                let (_, ref create_slot_t) = self.slots[&slot_id];
                                create_slot_t.send(new_slot);
                            },
                            SlotOutMsg::TransferClient(slot_id, client_id, new_slot_id) => {
                                match self.slots.get(&new_slot_id) {
                                    Some(slot) => {
                                        let &mut (ref mut client_slot_id, _) =
                                            client_outs.get_mut(&client_id).expect("Failed to get client output stream");
                                        if *client_slot_id == slot_id {
                                            let &(ref slot_in_t, _) = slot;
                                            *client_slot_id = new_slot_id; // set the client's new slot ID
                                            client_slots.get_mut(&client_id)
                                                .expect("Failed to get client slot")
                                                .clone_from(slot_in_t);
                                            slot_in_t.send(SlotInMsg::Joined(client_id));
                                        }
                                    },
                                    None => panic!("Failed to transfer client {} to non-existant slot {}", client_id, slot_id)
                                }
                            },
                        }
                    },
                    Err(_) => { break; }
                }
                
                if received_messages >= 10 {
                    break;
                }
            }
        }
    }
}

fn client_acceptor(listener: TcpListener, new_client_t: Sender<TcpStream>) {
    for stream in listener.incoming() {
        match stream {
            Err(e) => { println!("Incoming connection failed: {}", e); },
            Ok(stream) => {
                new_client_t.send(stream);
            }
        }
    }
}

fn handle_client_in(client_id: ClientId, mut stream: TcpStream, packet_in_t: Sender<(ClientId, InPacket)>) {
    loop {
        packet_in_t.send((client_id, InPacket::new_from_reader(&mut stream)));
    }
}

fn handle_client_out(mut stream: TcpStream, out_r: Receiver<OutPacket>) {
    loop {
        // Receive a packet to send
        let packet = 
            match out_r.recv() {
                Ok(packet) => packet,
                _ => panic!("Failed to receive out packet over channel."),
            };
        
        // Get the packet's data
        let data = packet.buffer.get_ref();
        
        // Write the packet size, then the actual packet data
        if let Err(e) = write_u16(&mut stream, data.len() as u16) {
            panic!("Failed to write packet length: {}", e);
        }
        if let Err(e) = stream.write(data) {
            panic!("Failed to write packet data: {}", e);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Client

pub struct Client {
    id: ClientId,
    stream: TcpStream,
    packet_receiver: Receiver<io::Result<InPacket>>,
}

impl Client {
    pub fn new(host: &str) -> Client {
        let mut stream = 
            match TcpStream::connect(host) {
                Ok(stream) => stream,
                Err(e) => panic!("Failed to connect to server: {}", e),
            };

        let id = 
            match read_u32(&mut stream) {
                Ok(id) => id,
                Err(e) => panic!("Couldn't connect to server because client ID failed to receive: {}", e),
            };
        
        let (packet_sender, packet_receiver) = channel();
        
        let mut thread_stream = stream.try_clone().ok().expect("Failed to clone client TcpStream");
        Builder::new().name("client_packet_receiver".to_string()).spawn(move || {
            loop {
                let packet = InPacket::try_new_from_reader(&mut thread_stream);
                packet_sender.send(packet);
            }
        });
    
        Client{id: id, stream: stream, packet_receiver: packet_receiver}
    }
    
    pub fn send(&mut self, packet: &OutPacket) {
        let data = &packet.buffer.get_ref();
        if let Err(e) = write_u16(&mut self.stream, data.len() as u16) {
            panic!("Failed to send packet size to server: {}", e);
        }
        match self.stream.write(&(*data)[..]) {
            Ok(bytes_written) => {
                if data.len() != bytes_written {
                    panic!("Failed to send packet data to server: Tried to write {} bytes, only {} bytes written", data.len(), bytes_written);
                }
            },
            Err(e) => { panic!("Failed to send packet data to server: {}", e); },
        }
    }
    
    pub fn receive(&mut self) -> InPacket {
        self.packet_receiver.recv()
            .ok().expect("Client packet sending channel closed")
            .ok().expect("Failed to receive packet from server")
    }
    
    pub fn try_receive(&mut self) -> io::Result<InPacket> {
        use std::io::{Error, ErrorKind};
        use std::sync::mpsc::TryRecvError;
    
        match self.packet_receiver.try_recv() {
            Ok(packet) => packet,
            Err(e) if e == TryRecvError::Empty =>
                Err(Error::new(ErrorKind::TimedOut, "No packet ready yet", None)),
            Err(_) =>
                Err(Error::new(ErrorKind::Other, "Client packet sending channel closed", None)),
        }
    }
    
    pub fn get_id(&self) -> ClientId {
        self.id
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Packet

#[derive(Clone)]
pub struct OutPacket {
    buffer: io::Cursor<Vec<u8>>,
}

impl OutPacket {
    pub fn new() -> OutPacket {
        OutPacket{buffer: io::Cursor::new(vec!())}
    }
    
    pub fn len(&self) -> usize {
        self.buffer.get_ref().len()
    }
    
    pub fn write<'a, T>(&mut self, t: &T) -> Result<(), EncodingError>
        where T: Encodable
    {
        encode_into(t, &mut self.buffer, SizeLimit::Infinite)
    }
}

pub struct InPacket {
    buffer: io::Cursor<Vec<u8>>,
}

impl InPacket {
    pub fn new(data: Vec<u8>) -> InPacket {
        InPacket{buffer: io::Cursor::new(data)}
    }
    
    pub fn new_from_reader<T: Read>(reader: &mut T) -> InPacket {
        // Get next packet size
        let packet_size =
            match read_u16(reader) {
                Err(e) => panic!("Failed to receive packet size: {}", e),
                Ok(packet_size) => packet_size
            };
        let packet_size = packet_size as u64;
        
        // Get data
        let mut data = vec!();
        match reader.take(packet_size).read_to_end(&mut data) {
            Err(e) => { panic!("Failed to receive data: {}", e); },
            Ok(bytes_read) =>
                if bytes_read as u64 != packet_size {
                    panic!("Expected {} bytes, got {} bytes", packet_size, bytes_read);
                },
        }
        
        // Build packet
        InPacket::new(data)
    }
    
    pub fn try_new_from_reader<T: Read>(reader: &mut T) -> io::Result<InPacket> {
        // Get next packet size
        let packet_size = try!(read_u16(reader));
        let packet_size = packet_size as u64;
    
        // Get data
        let mut data = vec!();
        let bytes_read = try!(reader.take(packet_size).read_to_end(&mut data));
        if bytes_read as u64 != packet_size {
            panic!("Expected {} bytes, got {} bytes", packet_size, bytes_read);
        }
        
        // Build packet
        Ok(InPacket::new(data))
    }
    
    pub fn len(&self) -> usize {
        self.buffer.get_ref().len()
    }
    
    pub fn read<T: Decodable>(&mut self) -> Result<T, DecodingError> {
        decode_from(&mut self.buffer, SizeLimit::Infinite)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn read_u16<T: Read>(reader: &mut T) -> io::Result<u16> {
    use std::io::{Error, ErrorKind};
    use std::mem;

    let mut buf: [u8; 2] = [0, 0];
    match reader.read(&mut buf) {
        Ok(bytes_read) => {
            if bytes_read == 2 {
                let data: u16 = unsafe { mem::transmute(buf) };
                Ok(data)
            } else {
                Err(Error::new(ErrorKind::Other, "Read incorrect number of bytes for u16", None))
            }
        },
        Err(e) =>{
            Err(e)
        },
    }
}

fn write_u16<T: Write>(writer: &mut T, data: u16) -> io::Result<usize> {
    use std::io::{Error, ErrorKind};
    use std::mem;

    let mut buf: [u8; 2] = unsafe { mem::transmute(data) };
    match writer.write(&buf) {
        Ok(bytes_written) => {
            if bytes_written == 2 {
                Ok(bytes_written)
            } else {
                Err(Error::new(ErrorKind::Other, "Wrote incorrect number of bytes for u16", None))
            }
        },
        Err(e) =>{
            Err(e)
        },
    }
}

fn read_u32<T: Read>(reader: &mut T) -> io::Result<u32> {
    use std::io::{Error, ErrorKind};
    use std::mem;

    let mut buf: [u8; 4] = [0, 0, 0, 0];
    match reader.read(&mut buf) {
        Ok(bytes_read) => {
            if bytes_read == 4 {
                let data: u32 = unsafe { mem::transmute(buf) };
                Ok(data)
            } else {
                Err(Error::new(ErrorKind::Other, "Read incorrect number of bytes for u32", None))
            }
        },
        Err(e) =>{
            Err(e)
        },
    }
}

fn write_u32<T: Write>(writer: &mut T, data: u32) -> io::Result<usize> {
    use std::io::{Error, ErrorKind};
    use std::mem;

    let mut buf: [u8; 4] = unsafe { mem::transmute(data) };
    match writer.write(&buf) {
        Ok(bytes_written) => {
            if bytes_written == 4 {
                Ok(bytes_written)
            } else {
                Err(Error::new(ErrorKind::Other, "Wrote incorrect number of bytes for u32", None))
            }
        },
        Err(e) => {
            Err(e)
        },
    }
}
