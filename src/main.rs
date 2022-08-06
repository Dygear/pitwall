#![allow(non_snake_case)]

use std::net::UdpSocket;
use colored::*;
use std::fmt;

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
struct Driver
{
    // From PacketParticipants.participants
    pub id: u8,                         // driverId
    pub idNetwork: u8,                  // networkId
    pub number: u8,                     // raceNumber
    pub nationality: u8,                // nationality
    pub isAI: bool,                     // aiControlled
    pub isTelemetryEnabled: bool,       // yourTelemetry
    pub name: String,                   // name
}

impl Driver
{
    pub fn getDriver(&self) -> String
    {
        format!(
            "{} ({:2})",
            if self.isAI { self.name.white() } else { self.name.yellow() },
            self.number,
        )
    }
}

#[derive(Debug, Default, Clone)]
struct Team
{
    // From PacketParticipants.participants
    pub id: u8,                         // teamId
    pub isCustom: bool,                 // myTeam
}

#[derive(Debug, Default, Clone)]
struct DRS
{
    // PacketCarTelemetry.carTelemetryData
    pub isOpen: bool,                   // drs
    // PacketCarStatus.carStatusData
    pub isAllowed: bool,                // drsAllowed
}

#[derive(Debug, Default, Clone)]
struct Assists
{
    // PacketCarStatus.carStatusData
    pub TC: TC,                         // tractionControl
    pub ABS: Assist,                    // antiLockBrakes
}

#[derive(Debug, Default, Clone)]
struct Tyres
{
    // PacketCarStatus.carStatusData
    pub actual: ActualCompound,         // actualTyre
    pub visual: VisualCompound,         // visualTyre
    pub age: u8,                        // tyresAgeLaps
}

impl fmt::Display for Tyres
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let esc = 27 as char;
        let reset    = "[0m";
        let _black   = "[30m";
        let red      = "[31m";
        let green    = "[32m";
        let yellow   = "[33m";
        let blue     = "[34m";
        let magenta  = "[35m";
        let _cyan    = "[36m";
        let white    = "[37m";

        match self.visual
        {
            VisualCompound::OldHard     | VisualCompound::Hard   => write!(f, "{esc}{}({}){esc}{reset}", white  , self.actual),
            VisualCompound::OldMedium   | VisualCompound::Medium => write!(f, "{esc}{}({}){esc}{reset}", yellow , self.actual),
            VisualCompound::OldSoft     | VisualCompound::Soft   => write!(f, "{esc}{}({}){esc}{reset}", red    , self.actual),
            VisualCompound::OldSuperSoft                         => write!(f, "{esc}{}({}){esc}{reset}", magenta, self.actual),
            VisualCompound::Inter                                => write!(f, "{esc}{}({}){esc}{reset}", green  , self.actual),
            VisualCompound::OldWet      | VisualCompound::Wet    => write!(f, "{esc}{}({}){esc}{reset}", blue   , self.actual),
                                                               _ => write!(f, "{}", self.actual)
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Telemetry
{
    // PacketCarTelemetry.carTelemetryData
    pub speed: KPH,                     // speed
    pub gear: Gear,                     // gear
    pub rpm: u16,                       // engineRPM
    pub leds: RevLights,                // revLightsBitValue
}

#[derive(Debug, Default, Clone)]
struct Time
{
    // PacketLap.laps
    pub sector1: TimeShort,             // sector1TimeInMS
    pub sector2: TimeShort,             // sector2TimeInMS
    pub sector3: TimeShort,             // sector2TimeInMS
    pub lastLap: TimeLong,              // lastLapTimeInMS
    pub current: TimeLong,              // currentLapTimeInMS
}

#[derive(Debug, Default, Clone)]
struct Car
{
    pub driver: Driver,
    pub team: Team,
    pub DRS: DRS,
    pub assist: Assists,
    pub tyres: Tyres,
    pub telemetry: Telemetry,
    pub time: Time,

    // PacketCarStatus.carStatusData
    pub underFlag: ZoneFlag,            // vehicleFiaFlags

    // PacketLap.laps
    pub spotGrid: u8,                   // gridPosition
    pub spotRace: u8,                   // carPosition
    pub lapNum: u8,                     // currentLapNum
    pub pitCount: u8,                   // numPitStops
    pub carStatus: CarState,            // driverStatus
    pub sector: u8,                     // sector

}

#[derive(Debug, Default, Clone, Copy)]
struct Lap {
    pub leader: u8,                     // MAX of PacketLap.laps.currentLapNum
    pub total: u8,                      // PacketSession.totalLaps
}

#[derive(Debug, Default, Clone, Copy)]
struct BestSector {
    pub time: TimeShort,                // MIN of PacketLap.laps.{sector1TimeInMS,sector2TimeInMS} Calc of sector3TimeInMS
    pub byId: u8,                       // Driver Index Number
    pub onLap: u8,                      // PacketLap.laps.currentLapNum
}

impl fmt::Display for BestSector
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.time.TimeInMS as f32 / 1000 as f32)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct BestLap {
    pub time: TimeLong,                 // MIN of PacketLap.laps.{lastLapTimeInMS}
    pub byId: u8,                       // Driver Index Number
    pub onLap: u8,                      // PacketLap.laps.currentLapNum
}

impl fmt::Display for BestLap
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.time.TimeInMS as f32 / 1000 as f32)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Bests {
    pub sector1: BestSector,
    pub sector2: BestSector,
    pub sector3: BestSector,
    pub lapTime: BestLap,
}

#[derive(Debug, Default, Clone)]
struct Page
{
    participants: u8,                   // PacketParticipants.numActiveCars
    positions: [u8; 22],
    r#type: Session,                    // PacketSession.sessionType
    bests: Bests,
    cars: [Car; 22],
    lap: Lap,
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
            Packet::Participants(p) => {
                page.participants = p.numActiveCars;

                for i in 0..22
                {
                    let idx = i as usize;

                    page.cars[idx].driver.isAI               = p.participants[idx].aiControlled == 1;
                    page.cars[idx].driver.id                 = p.participants[idx].driverId;
                    page.cars[idx].driver.idNetwork          = p.participants[idx].networkId;
                    page.cars[idx].team.id                   = p.participants[idx].teamId;
                    page.cars[idx].team.isCustom             = p.participants[idx].myTeam == 1;
                    page.cars[idx].driver.number             = p.participants[idx].raceNumber;
                    page.cars[idx].driver.nationality        = p.participants[idx].nationality;
                    page.cars[idx].driver.name               = p.participants[idx].name_to_string();
                    page.cars[idx].driver.isTelemetryEnabled = p.participants[idx].yourTelemetry == 1;
                }
            }
            Packet::CarTelemetry(t) => {
                for i in 0..22
                {
                    let idx = i as usize;

                    page.cars[idx].DRS.isOpen                = t.carTelemetry[idx].drs == 1;
                    page.cars[idx].telemetry.speed           = t.carTelemetry[idx].speed;
                    page.cars[idx].telemetry.gear            = t.carTelemetry[idx].gear;
                    page.cars[idx].telemetry.rpm             = t.carTelemetry[idx].engineRPM;
                    page.cars[idx].telemetry.leds            = t.carTelemetry[idx].revLightsBitValue;
                }

            }
            Packet::CarStatus(s) => {
                for i in 0..22
                {
                    let idx = i as usize;

                    page.cars[idx].DRS.isAllowed             = s.carStatus[idx].drsAllowed == 1;
                    page.cars[idx].assist.TC                 = s.carStatus[idx].tractionControl;
                    page.cars[idx].assist.ABS                = s.carStatus[idx].antiLockBrakes;
                    page.cars[idx].tyres.actual              = s.carStatus[idx].actualTyre;
                    page.cars[idx].tyres.visual              = s.carStatus[idx].visualTyre;
                    page.cars[idx].tyres.age                 = s.carStatus[idx].tyresAgeLaps;
                    page.cars[idx].underFlag                 = s.carStatus[idx].vehicleFiaFlags;
                }
            }
            Packet::Lap(l) => {
                for i in 0..22
                {
                    let idx = i as usize;
                    let pos = l.laps[idx].carPosition as usize;

                    // Update car positions.
                    page.positions[pos] = i;

                    // Update Leader Lap
                    page.lap.leader = {
                        if l.laps[idx].currentLapNum > page.lap.leader {
                            l.laps[idx].currentLapNum
                        } else {
                            page.lap.leader
                        }
                    };

                    // Sector 3 Time
                    if page.cars[idx].lapNum > l.laps[idx].currentLapNum {
                        page.cars[idx].time.sector3.TimeInMS = (
                            page.cars[idx].time.sector2.TimeInMS as u32 -
                            l.laps[idx].lastLapTimeInMS.TimeInMS
                        ) as u16
                    }

                    // Update Best Sectors
                    match page.cars[idx].sector
                    {
                        1 => {
                            // Check Sector 3 Time & Last Lap
                            if page.bests.sector3.time > page.cars[idx].time.sector3
                            {
                                page.bests.sector3 = BestSector {
                                    time: page.cars[idx].time.sector3,
                                    byId: i,
                                    onLap: l.laps[idx].currentLapNum - 1
                                }
                            }

                            // Last Lap
                            if page.bests.lapTime.time > page.cars[idx].time.lastLap
                            {
                                page.bests.lapTime = BestLap {
                                    time: page.cars[idx].time.lastLap,
                                    byId: i,
                                    onLap: l.laps[idx].currentLapNum - 1
                                }
                            }
                        },
                        2 => {
                            // Check Sector 1 Time
                            if page.bests.sector1.time > page.cars[idx].time.sector1
                            {
                                page.bests.sector1 = BestSector {
                                    time: page.cars[idx].time.sector1,
                                    byId: i,
                                    onLap: l.laps[idx].currentLapNum
                                }
                            }
                        },
                        3 => {
                            // Check Sector 2 Time
                            if page.bests.sector2.time > page.cars[idx].time.sector2
                            {
                                page.bests.sector2 = BestSector {
                                    time: page.cars[idx].time.sector2,
                                    byId: i,
                                    onLap: l.laps[idx].currentLapNum
                                }
                            }
                        },
                        _ => {}
                    }

                    // Now update the remaining new informaiton.
                    page.cars[idx].spotGrid                 = l.laps[idx].gridPosition;
                    page.cars[idx].spotRace                 = l.laps[idx].carPosition;
                    page.cars[idx].lapNum                   = l.laps[idx].currentLapNum;
                    page.cars[idx].pitCount                 = l.laps[idx].numPitStops;
                    page.cars[idx].carStatus                = l.laps[idx].driverStatus;
                    page.cars[idx].sector                   = l.laps[idx].sector;
                    page.cars[idx].time.sector1             = l.laps[idx].sector1TimeInMS;
                    page.cars[idx].time.sector2             = l.laps[idx].sector2TimeInMS;
                    page.cars[idx].time.current             = l.laps[idx].currentLapTimeInMS;
                    page.cars[idx].time.lastLap             = l.laps[idx].lastLapTimeInMS;
                }
            }
            _ => {
                continue;
            }
        }

        // Clear Screen & Corsor @ Top Left
        print!("{esc}c", esc = 27 as char);

        println!(
            "Lap: {lapLeader:02} {lapTotal:02}",
            lapLeader = page.lap.leader,
            lapTotal  = page.lap.total
        );

        // Header
            println!(
                "{spotRace:2} {driver:>24} {timeLastLap:>7} | {timeSector1:>7} {timeSector2:>7} {timeSector3:>7} | {timeCurrent:>7} {laps:>4} {tyre:>4} {status}",
                spotRace    = "P",
                driver      = "Driver",
                timeLastLap = "Last",
                timeSector1 = "S1",
                timeSector2 = "S2",
                timeSector3 = "S3",
                timeCurrent = "Time",
                laps        = "Laps",
                tyre        = "Tyre",
                status      = "Status",
            );

        // Bests
            println!(
                "{spotRace:2} {driver:>24} {bestLapTime:>7} | {bestSector1:>7} {bestSector2:>7} {bestSector3:>7} | {theoCurrent:>7} {laps:>4} {tyre:>4} {status}",
                spotRace    = "",
                driver      = "",
                bestLapTime = page.bests.lapTime,
                bestSector1 = page.bests.sector1,
                bestSector2 = page.bests.sector1,
                bestSector3 = page.bests.sector1,
                theoCurrent = "Time",
                laps        = "Laps",
                tyre        = "Tyre",
                status      = "Status",
            );

        // Drivers
        for i in 1..21
        {
            let p: usize = page.positions[i as usize].into();
            println!(
                "{spotRace:02} {driver:>33} {timeLastLap:>7} | {timeSector1:>7} {timeSector2:>7} {timeSector3:>7} | {timeCurrent:>7} {laps:>4} {tyre:#?} {status:#?}",
                spotRace    = page.cars[p].spotRace,
                driver      = page.cars[p].driver.getDriver(),
                timeLastLap = format!("{}", page.cars[p].time.lastLap),
                timeSector1 = format!("{}", page.cars[p].time.sector1),
                timeSector2 = format!("{}", page.cars[p].time.sector2),
                timeSector3 = format!("{}", page.cars[p].time.sector3),
                timeCurrent = format!("{}", page.cars[p].time.current),
                laps        = page.cars[p].lapNum,
                tyre        = page.cars[p].tyres,
                status      = page.cars[p].carStatus
            );
        }

        // Footer
        println!("");

    }
}
