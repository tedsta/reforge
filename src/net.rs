use std::io::{TcpListener, TcpStream, Acceptor, Listener};
use std::io::{MemReader, MemWriter};

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
        // Get next packet size
        let packet_size = stream.read_le_u16().unwrap();
        
        // Get data
        let packet = InPacket::new(stream.read_exact(packet_size as uint).unwrap());
        println!("Got: {}", packet_size);
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
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Packet

pub struct OutPacket {
    writer: MemWriter,
}

pub struct InPacket {
    reader: MemReader,
}

impl InPacket {
    pub fn new(data: Vec<u8>) -> InPacket {
        InPacket{reader: MemReader::new(data)}
    }
}