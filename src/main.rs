use std::str::FromStr;
use std::env;
mod miner;
mod block;
fn main() {

    // let block = block::Block::from_str(&block::Block::new(0, "Premier bloc".to_owned(), 0, 0, vec![0; 32]).to_string());
    // println!("{:?}", &block);

    let args: Vec<String> = env::args().collect();
    
    // On vérifie les arguments
    if args.len() == 1 {
        println!("miner: no operation specified");
        return ();
    } else if args.len() == 2 {
        println!("miner: must specify an IP and a port to join a network");
        return ();
    } else if args.len() > 4 {
        println!("miner: too many arguments specified");
        return ();
    }

    let role; // {--create, -c} pour créer un réseau, {--join, -j} pour en rejoindre un existant
    let socket = &args[2]; // L'ip:port du socket sur lequel le miner va écouter
    let mut address = &args[2];
    
    if &args[1] == "-c" || &args[1] == "--create" {
        role = "creator";
    } else if &args[1] == "-j" || &args[1] == "--join" {
        role = "joiner";
    } else {
        println!("miner: operation not recognized");
        return ();
    }

    if role == "joiner" {
        address = &args[3];
    }

    if role == "creator" {
        miner::create_miner('c',socket.to_string(), String::new());
    } else {
        miner::create_miner('j',socket.to_string(), address.to_string());
    }


    return ();
}
