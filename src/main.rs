#![allow(non_snake_case)]

use std::net::UdpSocket;
use colored::*;

mod packet;
use packet::*;

enum Packet
{
    Motion(PacketMotion),
    Session(PacketSession),
    Lap(PacketLap),
    Event(PacketEvent),
    Participants(PacketParticipants),
    CarSetups(PacketCarSetups),
    CarTelemetry(PacketCarTelemetry),
    CarStatus(PacketCarStatus),
    FinalClassification(PacketFinalClassification),
    LobbyInfo(PacketLobbyInfo),
    CarDamage(PacketCarDamage),
    SessionHistory(PacketSessionHistory),
    Unknown
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:20777").expect("Couldn't bind to address.");
    println!("UDP Port Bound");

    let mut buffer = [0; 4096];
    loop
    {
        let (size, _) = socket.recv_from(&mut buffer).unwrap();

        let header = Header::unpack(&buffer);
        
        match header.packetId {
            PacketId::Motion => {
                // Contains all motion data for player’s car – only sent while player is in control
                Packet::Motion(dbg!(PacketMotion::unpack(&buffer)))
            }
            PacketId::Session => {
                // Data about the session – track, time left
                Packet::Session(dbg!(PacketSession::unpack(&buffer)))
            }
            PacketId::Lap => {
                // Data about all the lap times of cars in the session
                Packet::Lap(dbg!(PacketLap::unpack(&buffer)))
            }
            PacketId::Event => {
                // Various notable events that happen during a session
                Packet::Event(dbg!(PacketEvent::unpack(&buffer)))
            }
            PacketId::Participants => {
                // List of participants in the session, mostly relevant for multiplayer
                Packet::Participants(dbg!(PacketParticipants::unpack(&buffer)))
            }
            PacketId::CarSetups => {
                // Packet detailing car setups for cars in the race
                Packet::CarSetups(dbg!(PacketCarSetups::unpack(&buffer)))
            }
            PacketId::CarTelemetry => {
                // Telemetry data for all cars
                Packet::CarTelemetry(dbg!(PacketCarTelemetry::unpack(&buffer)))
            }
            PacketId::CarStatus => {
                // Status data for all cars
                Packet::CarStatus(dbg!(PacketCarStatus::unpack(&buffer)))
            }
            PacketId::FinalClassification => {
                // Final classification confirmation at the end of a race
                Packet::FinalClassification(dbg!(PacketFinalClassification::unpack(&buffer)))
            }
            PacketId::LobbyInfo => {
                // Information about players in a multiplayer lobby
                Packet::LobbyInfo(dbg!(PacketLobbyInfo::unpack(&buffer)))
            }
            PacketId::CarDamage => {
                // Damage status for all cars
                Packet::CarDamage(dbg!(PacketCarDamage::unpack(&buffer)))
            }
            PacketId::SessionHistory => {
                // Lap and tyre data for session
                Packet::SessionHistory(dbg!(PacketSessionHistory::unpack(&buffer)))
            }
            PacketId::Poisoned => {
                dbg!(header);
                println!("{}, of {size}, & of ID {:#?}", "Unknown PacketId".red(), header.packetId);
                Packet::Unknown
            }
        };


    }
}
