#![allow(non_snake_case)]

use std::net::UdpSocket;
use prctl::set_name;
use colored::*;

mod packet;

fn main() {
    set_name("Timing and Scoring").expect("Couldn't set process title.");

    let socket = UdpSocket::bind("0.0.0.0:20777").expect("Couldn't bind to address.");
    println!("UDP Port Bound");

    let mut count: usize = 0;
    let mut buffer = [0; 4096];
    loop
    {
        count += 1;
        let (size, socketAddress) = socket.recv_from(&mut buffer).unwrap();
        println!("Got packet number {count} from {socketAddress} of size {size}.");

        let header = packet::Header::unpack(&buffer);
        
        match header.packetId {
            Some(packet::PacketId::Motion) => {
                dbg!(packet::PacketMotion::unpack(&buffer));
            },
            Some(packet::PacketId::SessionHistory) => {
                dbg!(packet::PacketSessionHistory::unpack(&buffer));
            }
            Some(_x) => {
                dbg!(header);
                println!("{} {:#?}", "Unhandled packetId".yellow(), header.packetId.unwrap());
            }
            None => {
                dbg!(header);
                println!("{} {:#?}", "Unknown PacketId".red(), header.packetId.unwrap());
            }
        };
    }
}
