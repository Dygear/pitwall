#![allow(non_snake_case)]

use std::net::UdpSocket;
use colored::*;
use std::fmt;

static ESC: char      = 27 as char;
static RESET: &str    = "[0m";
static _BLACK: &str   = "[30m";
static RED: &str      = "[31m";
static GREEN: &str    = "[32m";
static YELLOW: &str   = "[33m";
static BLUE: &str     = "[34m";
static MAGENTA: &str  = "[35m";
static _CYAN: &str    = "[36m";
static WHITE: &str    = "[37m";

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

    // PacketCarStatus.carStatusData
    pub underFlag: ZoneFlag,            // vehicleFiaFlags
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
        match self.visual
        {
            VisualCompound::OldHard     | VisualCompound::Hard   => write!(f, "{ESC}{color}({ESC}{RESET}{}{ESC}{color}){ESC}{RESET}", self.visual, color = WHITE  ),
            VisualCompound::OldMedium   | VisualCompound::Medium => write!(f, "{ESC}{color}({ESC}{RESET}{}{ESC}{color}){ESC}{RESET}", self.visual, color = YELLOW ),
            VisualCompound::OldSoft     | VisualCompound::Soft   => write!(f, "{ESC}{color}({ESC}{RESET}{}{ESC}{color}){ESC}{RESET}", self.visual, color = RED    ),
            VisualCompound::OldSuperSoft                         => write!(f, "{ESC}{color}({ESC}{RESET}{}{ESC}{color}){ESC}{RESET}", self.visual, color = MAGENTA),
            VisualCompound::Inter                                => write!(f, "{ESC}{color}({ESC}{RESET}{}{ESC}{color}){ESC}{RESET}", self.visual, color = GREEN  ),
            VisualCompound::OldWet      | VisualCompound::Wet    => write!(f, "{ESC}{color}({ESC}{RESET}{}{ESC}{color}){ESC}{RESET}", self.visual, color = BLUE   ),
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
    pub isSet: bool,                    // Has this been set yet?
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
    pub isSet: bool,                    // Has this been set yet?
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
    pub possible: BestLap,
}

#[derive(Debug, Default, Clone)]
struct Page
{
    participants: u8,                   // PacketParticipants.numActiveCars
    playerCarIndex: u8,                 // Always the last item, and so gives you the bounds of the array.
    positions: [usize; 22],
    r#type: Session,                    // PacketSession.sessionType
    bests: Bests,
    cars: [Car; 22],
    lap: Lap,
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:20777").expect("Couldn't bind to address.");
    println!("UDP Port Bound");

    let mut page = Page::default();

    let mut buffer = [0; 1500];
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
            Packet::Session(s) => {
                page.playerCarIndex = s.header.playerCarIndex;
                page.lap.total = s.totalLaps;
                page.r#type = s.sessionType;
            }
            Packet::Participants(p) => {
                page.participants = p.numActiveCars;

                for i in 0..=page.playerCarIndex
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
                for i in 0..=page.playerCarIndex
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
                for i in 0..=page.playerCarIndex
                {
                    let idx = i as usize;

                    page.cars[idx].DRS.isAllowed             = s.carStatus[idx].drsAllowed == 1;
                    page.cars[idx].assist.TC                 = s.carStatus[idx].tractionControl;
                    page.cars[idx].assist.ABS                = s.carStatus[idx].antiLockBrakes;
                    page.cars[idx].tyres.actual              = s.carStatus[idx].actualTyre;
                    page.cars[idx].tyres.visual              = s.carStatus[idx].visualTyre;
                    page.cars[idx].tyres.age                 = s.carStatus[idx].tyresAgeLaps;
                    page.cars[idx].driver.underFlag          = s.carStatus[idx].vehicleFiaFlags;
                }
            }
            Packet::Lap(l) => {
                for i in 0..=page.playerCarIndex
                {
                    let idx = i as usize;
                    let pos = l.laps[idx].carPosition as usize;

                    if pos == 0
                    {   // Skip empty slots.
                        continue
                    }

                    // Update car positions.
                    page.positions[pos] = idx;

                    // Update Leader Lap
                    page.lap.leader = {
                        if l.laps[idx].currentLapNum > page.lap.leader {
                            l.laps[idx].currentLapNum
                        } else {
                            page.lap.leader
                        }
                    };

                    // Sector 3 Time
                    if l.laps[idx].currentLapNum > page.cars[idx].lapNum
                    {
                        page.cars[idx].time.sector3 = TimeShort {
                            TimeInMS: (
                                    l.laps[idx].lastLapTimeInMS.TimeInMS - (
                                        page.cars[idx].time.sector1.TimeInMS as u32 +
                                        page.cars[idx].time.sector2.TimeInMS as u32
                                    )
                                ) as u16,
                            isPB: false,
                            isOB: false,
                        };
                    }

                    // Update Best Sectors
                    match l.laps[idx].sector
                    {
                        0 => {
                            // Check Sector 3
                            if l.laps[idx].currentLapNum > 1 && (!page.bests.sector3.isSet || page.bests.sector3.time > page.cars[idx].time.sector3)
                            {
                                page.bests.sector3 = BestSector {
                                    time: page.cars[idx].time.sector3,
                                    byId: i,
                                    onLap: l.laps[idx].currentLapNum,
                                    isSet: true,
                                };

                                page.cars[idx].time.sector3 = TimeShort {
                                    TimeInMS: page.cars[idx].time.sector3.TimeInMS,
                                    isPB: true,
                                    isOB: true,
                                };

                                // Check best.possible.
                                update_possible(&mut page.bests);
                            }

                            // Check best.lapTime.
                            if !page.bests.lapTime.isSet && l.laps[idx].currentLapNum != 1 || (page.bests.lapTime.time > l.laps[idx].lastLapTimeInMS)
                            {
                                page.bests.lapTime = BestLap {
                                    time: l.laps[idx].lastLapTimeInMS,
                                    byId: i,
                                    onLap: l.laps[idx].currentLapNum,
                                    isSet: true,
                                };

                                page.cars[idx].time.lastLap = TimeLong {
                                    TimeInMS: page.cars[idx].time.lastLap.TimeInMS,
                                    isPB: true,
                                    isOB: true,
                                };
                            }
                        },
                        1 => {
                            // Check Sector 1 Time
                            if !page.bests.sector1.isSet || page.bests.sector1.time > l.laps[idx].sector1TimeInMS
                            {
                                page.bests.sector1 = BestSector {
                                    time: l.laps[idx].sector1TimeInMS,
                                    byId: i,
                                    onLap: l.laps[idx].currentLapNum,
                                    isSet: true,
                                };

                                page.cars[idx].time.sector1 = TimeShort {
                                    TimeInMS: page.cars[idx].time.sector1.TimeInMS,
                                    isPB: true,
                                    isOB: true,
                                };

                                // Check best.possible.
                                update_possible(&mut page.bests);
                            }
                        },
                        2 => {
                            // Check Sector 2 Time
                            if !page.bests.sector2.isSet || page.bests.sector2.time > l.laps[idx].sector2TimeInMS
                            {
                                page.bests.sector2 = BestSector {
                                    time: l.laps[idx].sector2TimeInMS,
                                    byId: i,
                                    onLap: l.laps[idx].currentLapNum,
                                    isSet: true,
                                };

                                page.cars[idx].time.sector2 = TimeShort {
                                    TimeInMS: page.cars[idx].time.sector2.TimeInMS,
                                    isPB: true,
                                    isOB: true,
                                };

                                // Check best.possible.
                                update_possible(&mut page.bests);
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
        print!("{ESC}c");

        println!(
            "Lap: {lapLeader:02} {lapTotal:02}",
            lapLeader = page.lap.leader,
            lapTotal  = page.lap.total
        );

        // Header
            println!(
                "{idx:2} {pos:2} {driver:>24} {timeLastLap:>7} | {timeSector1:>7} {timeSector2:>7} {timeSector3:>7} | {timeCurrent:>7} | {lap:^3} {tyre:^4} {status:>6} {sector:>6} | {revLights:>15} {gear} {speed}",
                idx         = "ID",
                pos         = "P",
                driver      = "Driver",
                timeLastLap = "Last",
                timeSector1 = "S1",
                timeSector2 = "S2",
                timeSector3 = "S3",
                timeCurrent = "Time",
                lap         = "Lap",
                tyre        = "Tyre",
                status      = "Status",
                revLights   = "RevLights",
                gear        = "Gear",
                speed       = "KPH",
                sector      = "Sector"
            );

        // Cars
        // Need to figure out how to "zip" this together with:
        // let p: usize = page.positions[i as usize].into();
        // So that they are ordered correctly.
        for (pos, idx) in page.positions.iter().enumerate()
        {
            if pos == 0
            {   // Skip empty slots.
                continue
            }

            let car = &page.cars[*idx];

            println!(
                "{idx:02} {pos:02} {driver:>33} {timeLastLap:>16} | {timeSector1:>16} {timeSector2:>16} {timeSector3:>16} | {timeCurrent:>16} | {lap:^3} {tyre:^4}  {status:>6} {sector:>6} | {revLights} {gear:>4} {speed:>3}",
                driver      = car.driver.getDriver(),
                timeLastLap = format!("{}", car.time.lastLap),
                timeSector1 = format!("{}", car.time.sector1),
                timeSector2 = format!("{}", car.time.sector2),
                timeSector3 = format!("{}", car.time.sector3),
                timeCurrent = format!("{}", car.time.current),
                lap         = car.lapNum,
                tyre        = format!("{}", car.tyres),
                status      = format!("{}", car.carStatus),
                revLights   = car.telemetry.leds,
                gear        = format!("{}", car.telemetry.gear),
                speed       = format!("{}", car.telemetry.speed),
                sector      = car.sector
            );
        }

        // Footer
        println!("");

        // Bests
            println!(
                "{idx:2} {pos:2} {driver:>24} {bestLapTime:>7} | {bestSector1:>7} {bestSector2:>7} {bestSector3:>7} | {bestPossible:>7} | {lap:^3} {tyre:^4} {status:>6}",
                idx         = "",
                pos         = "",
                driver      = "Bests",
                bestLapTime = format!("{}", page.bests.lapTime),
                bestSector1 = format!("{}", page.bests.sector1),
                bestSector2 = format!("{}", page.bests.sector2),
                bestSector3 = format!("{}", page.bests.sector3),
                bestPossible= format!("{}", page.bests.possible),
                lap         = "",
                tyre        = "",
                status      = "",
            );

        println!("Bests: {{");
        let time =  page.bests.sector1.time.TimeInMS as f32 / 1000 as f32;
        println!("  sector1: {{time: TimeShort: {{TimeInMS: {time:>7}, isPB: {}, isOB: {}}}, byId: {:>2}, onLap: {}, isSet: {:>5}}},",   page.bests.sector1.time.isPB,  page.bests.sector1.time.isOB,  page.bests.sector1.byId,  page.bests.sector1.onLap,  page.bests.sector1.isSet);
        let time =  page.bests.sector2.time.TimeInMS as f32 / 1000 as f32;
        println!("  sector2: {{time: TimeShort: {{TimeInMS: {time:>7}, isPB: {}, isOB: {}}}, byId: {:>2}, onLap: {}, isSet: {:>5}}},",   page.bests.sector2.time.isPB,  page.bests.sector2.time.isOB,  page.bests.sector2.byId,  page.bests.sector2.onLap,  page.bests.sector2.isSet);
        let time =  page.bests.sector3.time.TimeInMS as f32 / 1000 as f32;
        println!("  sector3: {{time: TimeShort: {{TimeInMS: {time:>7}, isPB: {}, isOB: {}}}, byId: {:>2}, onLap: {}, isSet: {:>5}}},",   page.bests.sector3.time.isPB,  page.bests.sector3.time.isOB,  page.bests.sector3.byId,  page.bests.sector3.onLap,  page.bests.sector3.isSet);
        let time =  page.bests.lapTime.time.TimeInMS as f32 / 1000 as f32;
        println!("  lapTime: {{time:  TimeLong: {{TimeInMS: {time:>7}, isPB: {}, isOB: {}}}, byId: {:>2}, onLap: {}, isSet: {:>5}}},",   page.bests.lapTime.time.isPB,  page.bests.lapTime.time.isOB,  page.bests.lapTime.byId,  page.bests.lapTime.onLap,  page.bests.lapTime.isSet);
        let time = page.bests.possible.time.TimeInMS as f32 / 1000 as f32;
        println!(" possible: {{time:  TimeLong: {{TimeInMS: {time:>7}, isPB: {}, isOB: {}}}, byId: {:>2}, onLap: {}, isSet: {:>5}}},",  page.bests.possible.time.isPB, page.bests.possible.time.isOB, page.bests.possible.byId, page.bests.possible.onLap, page.bests.possible.isSet);
        println!("}}");

        // Footer
        println!("");

    }
}

fn update_possible(best: &mut Bests) -> bool
{
    if best.sector1.isSet && best.sector2.isSet && best.sector3.isSet
    {
        let newBestPossible: u32 = best.sector1.time.TimeInMS as u32 + best.sector2.time.TimeInMS as u32 + best.sector3.time.TimeInMS as u32;

        if !best.possible.isSet || best.possible.time.TimeInMS > newBestPossible
        {
            best.possible.time.TimeInMS = newBestPossible;
            return true;
        }
    }
    return false;
}