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
            PacketId::Motion => {
                // Contains all motion data for player’s car – only sent while player is in control
                dbg!(PacketMotion::unpack(&buffer));
            }
            PacketId::Session => {
                // Data about the session – track, time left
                dbg!(PacketSession::unpack(&buffer));
            }
            PacketId::Lap => {
                // Data about all the lap times of cars in the session
                dbg!(PacketLap::unpack(&buffer));
            }
            PacketId::Event => {
                // Various notable events that happen during a session
                // todo!()
                // dbg!(PacketEvent::unpack(&buffer));
            }
            PacketId::Participants => {
                // List of participants in the session, mostly relevant for multiplayer
                dbg!(PacketParticipants::unpack(&buffer));
            }
            PacketId::CarSetups => {
                // Packet detailing car setups for cars in the race
                dbg!(PacketCarSetup::unpack(&buffer));
            }
            PacketId::CarTelemetry => {
                // Telemetry data for all cars
                dbg!(PacketCarTelemetry::unpack(&buffer));
            }
            PacketId::CarStatus => {
                // Status data for all cars
                dbg!(PacketCarStatus::unpack(&buffer));
            }
            PacketId::FinalClassification => {
                // Final classification confirmation at the end of a race
                dbg!(PacketFinalClassification::unpack(&buffer));
            }
            PacketId::LobbyInfo => {
                // Information about players in a multiplayer lobby
                dbg!(PacketLobbyInfo::unpack(&buffer));
            }
            PacketId::CarDamage => {
                // Damage status for all cars
                dbg!(PacketCarDamage::unpack(&buffer));
            }
            PacketId::SessionHistory => {
                // Lap and tyre data for session
                dbg!(PacketSessionHistory::unpack(&buffer));
            }
            PacketId::Poisoned => {
                dbg!(header);
                println!("{}, of {size}, & of ID {:#?}", "Unknown PacketId".red(), header.packetId);
            }
        };
    }
}
