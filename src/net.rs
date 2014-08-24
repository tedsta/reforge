use std::io::{TcpListener, TcpStream, Acceptor, Listener};
use std::io::{MemReader, MemWriter, IoResult};

use std::collections::HashMap;

use rsfml::network::Packet;

///////////////////////////////////////////////////////////////////////////////////////////////////
// Server

pub struct Server {
    listener: TcpListener,
    
    // Packet receivers
    receivers: HashMap<u32, Receiver<Packet>>,
}

impl Server {
    pub fn new(port: u16) -> Server {
        Server{listener: TcpListener::bind("127.0.0.1", port).ok().unwrap(), receivers: HashMap::new()}
    }
    
    pub fn set_default_receiver(&mut self) {
    }
    
    pub fn listen(self) {
        let mut acceptor = self.listener.listen();

        // accept connections and process them, spawning a new tasks for each one
        for stream in acceptor.incoming() {
            match stream {
                Err(e) => println!("Incoming connection failed: {}", e),
                Ok(stream) => spawn(proc() {
                    // connection succeeded
                    handle_client(stream)
                })
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    loop {
        // Build packet
        let mut packet = InPacket::new_from_reader(&mut stream);
        
        println!("{}, {}, {}", packet.read_int().unwrap(), packet.read_uint().unwrap(), packet.read_int().unwrap());
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Client

pub struct Client {
    stream: TcpStream,
    
    // Receiver ID on server to send packets to
    send_to: u32,
}

impl Client {
    pub fn new(host: &str, port: u16) -> Client {
        Client{stream: TcpStream::connect(host, port).unwrap(), send_to: 0}
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