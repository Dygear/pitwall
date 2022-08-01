#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::{
    mem::{
        size_of
    },
    net::{
        UdpSocket
    }
};
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
            Some(packet::PacketId::Lap) => {
//                let lapPack = packet::PacketLap::unpack(&buffer);
//                dbg!(lapPack);
            }
            Some(packet::PacketId::Participants) => {
                let participantsPack = packet::PacketParticipants::unpack(&buffer);
                dbg!(participantsPack);
            }
            Some(_x) => {
                println!("{} {:#?}", "Unhandled packetId".yellow(), header.packetId.unwrap());
            }
            None => {
                dbg!(header);
                println!("{} {:#?}", "Unknown PacketId".red(), header.packetId.unwrap());
            }
        };
    }
}
