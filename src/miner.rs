use std::net::{TcpStream, TcpListener, Shutdown};
use std::fmt::{self, Debug, Formatter};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::str::FromStr;
use crossbeam_utils::thread;
use std::collections::HashSet;

#[path="./block.rs"]
mod block;

#[derive(Copy, Clone)]
pub enum Flag {
    /// Ok -> Network
    Ok,
    Connect, // flag to signal that a Miner joined the newtwork
    Disconnect, // flag to signal that a Miner disconnected from the network
    RequireID,
    GiveID,
    BroadcastConnect,
    BroadcastDisconnect
}

impl Flag {
    fn from_u8(value: u8) -> Flag {
        match value {
            0 => Flag::Ok,
            1 => Flag::Connect,
            2 => Flag::Disconnect,
            3 => Flag::RequireID,
            4 => Flag::GiveID,
            5 => Flag::BroadcastConnect,
            6 => Flag::BroadcastDisconnect,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

/// Pimped serialized hashset 
/// 
/// *`set` The hashSet to serialized 
pub fn hashset_to_string(set: &HashSet<(u32, String)>) -> String {
    let mut res = vec![];
    for (id, addr) in set {
        res.push(id.to_string() +","+ &addr.to_string());
        println!("{}, {}",id,addr);
    }
    res.join(";")
}

/// Pimped deserialized hashset
/// 
/// 
pub fn hashset_from_string(hashset :String) -> HashSet<(u32, String)> {
    let mut res = HashSet::<(u32,String)>::new();
    let splitted: Vec<&str> = hashset.split(";").collect();
    println!("{}", splitted[0]);
    for element in splitted {
        let couple: Vec<&str> = element.split(",").collect();
        let id: u32 =  String::from(couple[0].to_string().trim_matches(char::from(0))).parse::<u32>().unwrap();
        let address: String = couple[1].to_string();
        println!("id:{}, addr: {}", &id, &address);
        res.insert((id,address));
    }
    return res.to_owned();
}

/// Util
/// Conctene u8 array
/// * `first`
/// * `second`
pub fn concat_u8(first: &[u8], second: &[u8]) -> Vec<u8> {
    let tmp = [first, second].concat();
    tmp
}
// Ajoute du padding sockip
pub fn encode_sockip(sockip: String) -> String {
    return format!("{:X<21}", sockip);
}

// Retire le padding au sockip
pub fn decode_sockip(sockip: String) -> String {
    return str::replace(&sockip, "X", "");
}

pub fn encode_id(id: String) -> String {
    return format!("{:Y<10}", id);
}

pub fn decode_id(id: String) -> String {
    return str::replace(&id, "Y", "");
}


pub fn decode_message(msg : &[u8]) -> (Flag, String, String, String){
    println!("SSSDL111 .{:?}.", msg);
    let flag = Flag::from_u8(msg[0]); // get the flag
    let sockip_encoded = std::str::from_utf8(&msg[1..21]).unwrap();
    let id_encoded = std::str::from_utf8(&msg[22..31]).unwrap();
    let msg = std::str::from_utf8(&msg[32..]).unwrap();
    let sockip = decode_sockip(sockip_encoded.to_string());
    (flag, decode_sockip(sockip.to_string()), decode_id(id_encoded.to_string()), msg.to_string())
}

pub fn encode_message(flag : &Flag, sockip : String, id : String, msg : String) -> Vec<u8>{
    let flag_convert = *flag as u8;
    let sockip_convert : String = encode_sockip(sockip);
    let id_convert : String = encode_id(id);
    let msg_convert : &[u8] = msg.as_bytes();
    concat_u8(&[flag_convert], &concat_u8(sockip_convert.as_bytes(), &concat_u8(id_convert.as_bytes(), msg_convert)))
}

pub fn create_miner(miner_type: char, socket: String, destination: String) {
    println!("Miner creation...");
    let mut miner;
    match miner_type {
        'c' => { miner = Miner::new(0, socket.to_string()); }
        'j' => { miner = Miner::new(ask_for_id(&socket, &destination), socket.to_string()); }
        _ => { println!("Unrecognized miner type"); return (); }
    }
    miner.add_to_network(miner.get_id(),socket.to_string());
    println!("{:?}", &miner);
    for (i,e) in &miner.network {
        println!("{}, {}",i,e);
    }
    if !!! destination.is_empty() {
        println!("Now connecting to network...");
        miner.join(destination);
        println!("Connected!\n");
    }
    println!("Starting to listen...");
    miner.listen();
}

pub fn ask_for_id(socket: &String, destination: &String) -> u32 {
    println!("Asking {} for id...", &destination);
    let listener = TcpListener::bind(&socket).unwrap();
    let mut id: u32 = 0;

    if let Ok(mut stream) = TcpStream::connect(&destination) {
        let m: &[u8] = &encode_message(&Flag::RequireID, socket.to_string(), "".to_string(), "".to_string());
        match stream.write(m) {
            Ok(_) => { println!("Asked for id"); }
            Err(e) => { println!("Error: {}", e); }
        }
        println!("Message sended");
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Getting ID from Genesis");
                id = handle_id(stream);
                println!("My ID is {}.", &id);
                return id;
            }
            Err(e) => {
                println!("Error: {}", e);
                return 0;
            }
        }
    }
    return id;
} 

pub fn handle_id(mut stream: TcpStream) -> u32 {
    let mut data = [0 as u8; 50];
    match stream.read(&mut data) {
        Ok(size) if size > 0 => {
            let id_as_str_decoded = decode_id(std::str::from_utf8(&data[32..size]).unwrap().to_owned());
            let id = id_as_str_decoded.parse::<u32>().unwrap();
            return id;
        },
        Ok(_) => { println!("No message received");},
        Err(e) => {
            println!("Error occured, closing connection: {}", e);
            stream.shutdown(Shutdown::Both).unwrap();
        }
    }
    {}
    0
}

pub struct Miner {
    pub id: u32, // Our ID
    pub network: HashSet<(u32, String)>, // The IDs of every member of the network, always unique
    pub blocks: Vec<block::Block>, // The blocks calculated by us
    pub sockip: String,
}

impl Miner {

    /// CONSTRUCTOR
    /// `socket` - an ip:port string representing where is the Miner listening
    /// returns a new Miner with a TcpListener that listens to the given ip:port
    pub fn new (id: u32, socket: String) -> Self {
        return Miner {
            id: id,
            network: HashSet::new(),
            blocks: Vec::new(),
            sockip: socket.to_string(),
        }        
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// Function to join an existing network
    /// * `destination` - the ip:port of the Miner we want to join
    pub fn join(&self, destination: String) {
        // Connexion au socket distant
        self.send_message(&destination, &self.id.to_string(), &Flag::Connect);
        println!("Join done.");
    }

    
    /// Function to send a message
    /// * `stream` - Tcp Stream.
    /// * `message` - The message to send.
    pub fn send_message(&self, destination: &String, message: &String, flag: &Flag) {
        println!("Sending message: {} \nTo: {}",&message, &destination);
        if let Ok(mut stream) = TcpStream::connect(&destination) {
            let m: &[u8] = &encode_message(flag, self.sockip.to_string(), self.id.to_string(), message.to_string());
            stream.write(m);
            println!("Message sended");
        } else {
            println!("Connection with {} failed.", &destination);
        }
    }

    pub fn broadcast_to_network(&self, message: &String, flag: Flag, sender: String) {
        println!("Broadcasting network changes");
        for(_, peer_addr) in &self.network {
            if peer_addr.to_string() != sender {
                self.send_message(&peer_addr, message, &flag);
            }
        }
    }

    /// Message propagation to all neighbors
    /// * `message` - Message sent.
    /// Unused
    pub fn broadcast_threaded(&self, message: &String) {
        // For each neighbor
        println!("Broadcasting the message {}", &message);
        for (_, neighbor_address) in &self.network {
            // Open connection with another thread
            thread::scope(|s| {
                s.spawn(move |_| {
                    // Connect to neighbor             
                    self.send_message(&neighbor_address, &message, &Flag::Ok); // TODO : Change Flag
                });
            });
        }
    }

    pub fn retrieve_next_id(&self) -> u32 {
        let mut max_id = &self.id;
        for (id, _) in &self.network {
            if id > max_id {
                max_id = id;
            }
        }
        println!("found id is {}", max_id);
        return (max_id+1).to_owned();
    }

    pub fn handle_client(&mut self, mut stream: TcpStream) {
        let mut data = [0 as u8; 50];
        while match stream.read(&mut data) { 
            Ok(size) if size > 0 => { // If a message is received
                println!("Message received of size: {}", &size);
                let tuple : (Flag, String, String, String) = decode_message(&data);
                //let flag = Flag::from_u8(data[0]); // get the flag
                let flag = tuple.0;
                println!("\tFlag: {}", &data[0]);
                //println!("\tFlag: {:?}", &flag);
                //let message = std::str::from_utf8(&data[0..size]).unwrap();
                let message = tuple.3;
                println!("\tMessage: {}", &message);

                let text = &message[1..]; // get the remainder of the message
                let sender_sockip = tuple.1;
                println!("\tSockIp: {}", &sender_sockip);

                let sender_id_as_str = tuple.2;
                

                // select appropriate response based on the flag, convert the u8 number to flag
                match flag {
                    Flag::Connect => {
                        println!("OK!");
                        //let destination = format!("{}:{}",&stream.local_addr().unwrap().ip().to_string(),&stream.local_addr().unwrap().port().to_string());
                        let destination = &sender_sockip;
                        self.send_message(&destination , &hashset_to_string(&self.network), &Flag::Ok);
                        println!("{}", sender_id_as_str);
                        let sender_id = sender_id_as_str.parse::<u32>().unwrap();
                        let broadcast_message = format!("{};{}", sender_sockip, sender_id);
                        self.broadcast_to_network(&broadcast_message, Flag::BroadcastConnect, self.sockip.to_string());
                        self.add_to_network(sender_id, sender_sockip);
                    }
                    Flag::Disconnect => {
                        let sender_id = sender_id_as_str.parse::<u32>().unwrap();
                        self.remove_from_network(sender_id, sender_sockip.to_owned());
                        let broadcast_message = format!("{};{}", sender_id, sender_id);
                        self.broadcast_to_network(&message, Flag::BroadcastDisconnect, self.sockip.to_string());
                    }
                    Flag::Ok => {
                        let received_network = message;
                        println!("Reply is ok!\nNetwork:{} \n {}", &received_network, &received_network.chars().count());
                        println!("{} ",&received_network.chars().nth(40).unwrap_or('0'));                     

                        let network: HashSet<(u32,String)> = hashset_from_string(received_network);
                        for (i,e) in &network {
                            println!("{}, {}",i,e);
                        }
                        // self.network | network;
                        self.network = self.network.union(&network).into_iter().cloned().collect::<HashSet<_>>();
                        println!("New network: ");
                        for (i,e) in &self.network {
                            println!("{}, {}",i,e);
                        }
                    }
                    Flag::RequireID => {
                        let next_id = self.retrieve_next_id().to_string();
                        self.send_message(&sender_sockip, &next_id, &Flag::GiveID);
                    }
                    Flag::BroadcastConnect => {

                        let splitted: Vec<&str> = message.split(";").collect();
                        let new_sockip = &splitted[0].to_string();
                        let new_id_as_str = &splitted[1].to_string();
                        println!("id:{}, sockip:{}", new_id_as_str.to_string(), new_sockip.to_string());
                        println!("The message is: -{}-", &message);
                        let new_id = new_id_as_str.parse::<u32>().unwrap();
                        
                        if self.add_to_network(new_id, new_sockip.to_string()) {
                            self.broadcast_to_network(&message, Flag::BroadcastConnect, sender_sockip);
                        }
                    }
                    _ => { println!("Error: flag not recognized"); }
                } 
                data = [0 as u8; 50];
                true
            },
            Ok(_) => { println!("No message received"); false },
            Err(e) => {
                println!("Error occurs, closing connection: {}", e);
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        }
        {}       
    }

    /// Function to add a Miner to the network
    /// `peer_id` - an integer to identify the Miner, should be unique in the network
    /// `peer_addr` - the socket on which the Miner is listening, should be unique aswell
    /// Update the current Miner's network, returns true if the Miner was added to the newtork, false if the Miner was already in the network
    pub fn add_to_network(&mut self, peer_id: u32, peer_addr: String) -> bool {
        self.network.insert((peer_id, peer_addr))
    }

    /// Function to remove a Miner from the network
    /// `peer_id` - an integer to identify the Miner
    /// `peed_addr` - the socket of the Miner we want to remove from the network
    /// Update the current Miner's network, returns true if the Miner was deleted from the newtork, false if the Miner wasn't in the network
    pub fn remove_from_network(&mut self, peer_id: u32, peer_addr: String) -> bool {
        self.network.remove(&(peer_id, peer_addr))
    }

    /// Function to listen for incoming Streams from the network
    /// Read the stream and spawn a thread to handle the received data
    pub fn listen(mut self) {
        println!("Server listening on port {}", &self.sockip);
        let listener = TcpListener::bind(&self.sockip).unwrap();
        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", &stream.peer_addr().unwrap());  
                    self.handle_client(stream);
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
            println!("\nCurrent network:");
            for miner in &self.network {
                println!("\tid: {}, sockip: {}", miner.0, miner.1);
            }
        }
        // close the socket server
        drop(listener);
    }

}

impl Debug for Miner {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Miner[{}]: \n Network:",
            &self.id,
            //&self.network,
        )
        
    }
}
