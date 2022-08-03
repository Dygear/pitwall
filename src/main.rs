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

#[derive(Debug, Default, Clone)]
struct Info
{
    // From PacketParticipants.participants
    pub aiControlled: bool,             // aiControlled
    pub driverId: u8,                   // driverId
    pub networkId: u8,                  // networkId
    pub teamId: u8,                     // teamId
    pub teamIsCustom: bool,             // myTeam
    pub carNumber: u8,                  // raceNumber
    pub driverNationality: u8,          // nationality
    pub driverName: String,             // name
    pub driverTelemetry: bool,          // yourTelemetry
    // PacketCarTelemetry.carTelemetryData
    pub isDRSOpen: bool,                // drs
    pub speed: KPH,                     // speed
    pub gear: Gear,                     // gear
    pub rpm: u16,                       // engineRPM
    pub leds: RevLights,                // revLightsBitValue
    // PacketCarStatus.carStatusData
    pub isDRSAllowed: bool,             // drsAllowed
    pub assistTC: TC,                   // tractionControl
    pub assistABS: Assist,              // antiLockBrakes
    pub tyreActual: ActualCompound,     // actualTyre
    pub tyreVisual: VisualCompound,     // visualTyre
    pub tyreAge: u8,                    // tyresAgeLaps
    pub underFlag: ZoneFlag,            // vehicleFiaFlags
    // PacketLap.laps
    pub spotGrid: u8,                   // gridPosition
    pub spotRace: u8,                   // carPosition
    pub lapNum: u8,                     // currentLapNum
    pub pitCount: u8,                   // numPitStops
    pub carStatus: Driver,              // driverStatus
    pub sector: u8,                     // sector
    pub timeSector1: TimeShort,         // sector1TimeInMS
    pub timeSector2: TimeShort,         // sector2TimeInMS
    pub timeCurrent: TimeLong,          // currentLapTimeInMS
    pub timeLastLap: TimeLong,          // lastLapTimeInMS
}
/**
#[derive(Debug, Default, Clone, Copy)]
struct Laps {
    lead: u8,                           // MAX of PacketLap.laps.currentLapNum
    total: u8,                          // PacketSession.totalLaps
}

#[derive(Debug, Default, Clone, Copy)]
struct BestSector {
    pub time: TimeShort,                // MIN of PacketLap.laps.{sector1TimeInMS,sector2TimeInMS} Calc of sector3TimeInMS
    pub onLap: u8,                      // PacketLap.laps.currentLapNum
    pub by: u8,                         // Driver Index Number
}

#[derive(Debug, Default, Clone, Copy)]
struct BestLap {
    pub time: TimeLong,                 // MIN of PacketLap.laps.{lastLapTimeInMS}
    pub onLap: u8,                      // PacketLap.laps.currentLapNum
    pub by: u8,                         // Driver Index Number
}

#[derive(Debug, Default, Clone, Copy)]
struct Bests {
    pub One: BestSector,
    pub Two: BestSector,
    pub Three: BestSector,
    pub Lap: BestLap,
}
*/
#[derive(Debug, Default, Clone)]
struct Page
{
    drivers: [Info; 22],
    participants: u8,                   // PacketParticipants.numActiveCars
    r#type: Session,                    // PacketSession.sessionType
    positions: [u8; 22],
//    lap: Laps,
//    best: Bests
}

impl std::fmt::Display for Page
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Format: {})", self.drivers.len())
    }
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:20777").expect("Couldn't bind to address.");
    println!("UDP Port Bound");

    let mut page = Page::default();

    let mut buffer = [0; 4096];
    loop
    {
        let (size, _) = socket.recv_from(&mut buffer).unwrap();

        let header = Header::unpack(&buffer);
        
        let packet = match header.packetId {
            PacketId::Motion => {
                // Contains all motion data for player’s car – only sent while player is in control
                Packet::Motion(PacketMotion::unpack(&buffer))
            }
            PacketId::Session => {
                // Data about the session – track, time left
                Packet::Session(PacketSession::unpack(&buffer))
            }
            PacketId::Lap => {
                // Data about all the lap times of cars in the session
                Packet::Lap(PacketLap::unpack(&buffer))
            }
            PacketId::Event => {
                // Various notable events that happen during a session
                Packet::Event(PacketEvent::unpack(&buffer))
            }
            PacketId::Participants => {
                // List of participants in the session, mostly relevant for multiplayer
                Packet::Participants(PacketParticipants::unpack(&buffer))
            }
            PacketId::CarSetups => {
                // Packet detailing car setups for cars in the race
                Packet::CarSetups(PacketCarSetups::unpack(&buffer))
            }
            PacketId::CarTelemetry => {
                // Telemetry data for all cars
                Packet::CarTelemetry(PacketCarTelemetry::unpack(&buffer))
            }
            PacketId::CarStatus => {
                // Status data for all cars
                Packet::CarStatus(PacketCarStatus::unpack(&buffer))
            }
            PacketId::FinalClassification => {
                // Final classification confirmation at the end of a race
                Packet::FinalClassification(PacketFinalClassification::unpack(&buffer))
            }
            PacketId::LobbyInfo => {
                // Information about players in a multiplayer lobby
                Packet::LobbyInfo(PacketLobbyInfo::unpack(&buffer))
            }
            PacketId::CarDamage => {
                // Damage status for all cars
                Packet::CarDamage(PacketCarDamage::unpack(&buffer))
            }
            PacketId::SessionHistory => {
                // Lap and tyre data for session
                Packet::SessionHistory(PacketSessionHistory::unpack(&buffer))
            }
            PacketId::Poisoned => {
                dbg!(header);
                println!("{}, of {size}, & of ID {:#?}", "Unknown PacketId".red(), header.packetId);
                Packet::Unknown
            }
        };

        match packet
        {
            Packet::Session(session) => {
                page.r#type = session.sessionType;
            }
            Packet::Participants(people) => {
                page.participants = people.numActiveCars;

                for i in 0..22
                {
                    page.drivers[i as usize].aiControlled      = people.participants[i as usize].aiControlled == 1;
                    page.drivers[i as usize].driverId          = people.participants[i as usize].driverId;
                    page.drivers[i as usize].networkId         = people.participants[i as usize].networkId;
                    page.drivers[i as usize].teamId            = people.participants[i as usize].teamId;
                    page.drivers[i as usize].teamIsCustom      = people.participants[i as usize].myTeam == 1;
                    page.drivers[i as usize].carNumber         = people.participants[i as usize].raceNumber;
                    page.drivers[i as usize].driverNationality = people.participants[i as usize].nationality;
                    page.drivers[i as usize].driverName        = people.participants[i as usize].name_to_string();
                    page.drivers[i as usize].driverTelemetry   = people.participants[i as usize].yourTelemetry == 1;
                }
            }
            Packet::CarTelemetry(telemetry) => {
                for i in 0..22
                {
                    page.drivers[i as usize].isDRSOpen = telemetry.carTelemetry[i as usize].drs == 1;
                    page.drivers[i as usize].speed     = telemetry.carTelemetry[i as usize].speed;
                    page.drivers[i as usize].gear      = telemetry.carTelemetry[i as usize].gear;
                    page.drivers[i as usize].rpm       = telemetry.carTelemetry[i as usize].engineRPM;
                    page.drivers[i as usize].leds      = telemetry.carTelemetry[i as usize].revLightsBitValue;
                }

            }
            Packet::CarStatus(status) => {
                for i in 0..22
                {
                    page.drivers[i as usize].isDRSAllowed = status.carStatus[i as usize].drsAllowed == 1;
                    page.drivers[i as usize].assistTC     = status.carStatus[i as usize].tractionControl;
                    page.drivers[i as usize].assistABS    = status.carStatus[i as usize].antiLockBrakes;
                    page.drivers[i as usize].tyreActual   = status.carStatus[i as usize].actualTyre;
                    page.drivers[i as usize].tyreVisual   = status.carStatus[i as usize].visualTyre;
                    page.drivers[i as usize].tyreAge      = status.carStatus[i as usize].tyresAgeLaps;
                    page.drivers[i as usize].underFlag    = status.carStatus[i as usize].vehicleFiaFlags;
                }
            }
            Packet::Lap(lap) => {
                for i in 0..22
                {
                    page.positions[lap.laps[i as usize].carPosition as usize] = i;

                    page.drivers[i as usize].spotGrid    = lap.laps[i as usize].gridPosition;
                    page.drivers[i as usize].spotRace    = lap.laps[i as usize].carPosition;
                    page.drivers[i as usize].lapNum      = lap.laps[i as usize].currentLapNum;
                    page.drivers[i as usize].pitCount    = lap.laps[i as usize].numPitStops;
                    page.drivers[i as usize].carStatus   = lap.laps[i as usize].driverStatus;
                    page.drivers[i as usize].sector      = lap.laps[i as usize].sector;
                    page.drivers[i as usize].timeSector1 = lap.laps[i as usize].sector1TimeInMS;
                    page.drivers[i as usize].timeSector2 = lap.laps[i as usize].sector2TimeInMS;
                    page.drivers[i as usize].timeCurrent = lap.laps[i as usize].currentLapTimeInMS;
                    page.drivers[i as usize].timeLastLap = lap.laps[i as usize].lastLapTimeInMS;
                }
            }
            _ => {
                continue;
            }
        }

        // Clear Screen & Corsor @ Top Left
        print!("{esc}c", esc = 27 as char);

        // Header
        println!(
            "{spotRace:02} {carNumber:02} {driverName:>20} {timeCurrent:>7} | {timeSector1:>7} {timeSector2:>7} | {timeLastLap:>7} {laps:>4} {tyre:>4} {status}",
            spotRace    = "##",
            carNumber   = "##",
            driverName  = "Driver",
            timeCurrent = "Time",
            timeSector1 = "S1",
            timeSector2 = "S2",
            timeLastLap = "Last",
            laps        = "Laps",
            tyre        = "Tyre",
            status      = "Status",
        );

        // Drivers
        for i in 0..22
        {
            let p: usize = page.positions[i as usize].into();
            println!(
                "{spotRace:02} {carNumber:02} {driverName:>20} {timeCurrent:>7} | {timeSector1:>7} {timeSector2:>7} | {timeLastLap:>7} {laps:>4} {tyre:#?} {status:#?}",
                spotRace    = page.drivers[p].spotRace,
                carNumber   = page.drivers[p].carNumber,
                driverName  = page.drivers[p].driverName,
                timeCurrent = format!("{}", page.drivers[p].timeCurrent),
                timeSector1 = format!("{}", page.drivers[p].timeSector1),
                timeSector2 = format!("{}", page.drivers[p].timeSector2),
                timeLastLap = format!("{}", page.drivers[p].timeLastLap),
                laps        = page.drivers[p].lapNum,
                tyre        = page.drivers[p].tyreActual,
                status      = page.drivers[p].carStatus
            );
        }

        println!("");

    }
}
