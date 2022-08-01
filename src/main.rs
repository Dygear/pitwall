#![allow(non_snake_case)]

use std::net::UdpSocket;
use prctl::set_name;
use colored::*;

mod packet;
use packet::*;

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

        let header = Header::unpack(&buffer);
        
        match header.packetId {
            PacketId::Lap => {
                dbg!(PacketLap::unpack(&buffer));
            }
            PacketId::Participants => {
                dbg!(PacketParticipants::unpack(&buffer));
            }
            PacketId::Unknown => {
                dbg!(header);
                println!("{}, of {size}, & of ID {:#?}", "Unknown PacketId".red(), header.packetId);
            }
            other => {
                println!("{} {:#?}", "Unhandled packetId".yellow(), other);
                unreachable!(); // Theoretically
            }
        };
    }
}
