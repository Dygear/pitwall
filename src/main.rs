#![allow(non_snake_case)]

use std::net::UdpSocket;
use colored::*;
use std::fmt;

static ESC: char      = 27 as char;

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
            "{:>15} ({:2})",
            match self.underFlag {
                ZoneFlag::Green => {
                    if self.isAI
                    {
                        self.name.white().on_green()
                    }
                    else
                    {
                        self.name.yellow().on_green()
                    }
                },
                ZoneFlag::Blue => {
                    if self.isAI
                    {
                        self.name.white().on_blue()
                    }
                    else
                    {
                        self.name.yellow().on_blue()
                    }
                },
                ZoneFlag::Yellow => {
                    if self.isAI
                    {
                        self.name.white().on_yellow()
                    }
                    else
                    {
                        self.name.black().on_yellow()
                    }
                },
                _ => {
                    if self.isAI
                    {
                        self.name.white()
                    }
                    else
                    {
                        self.name.yellow()
                    }
                },
            },
            self.number
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
struct Drs
{
    // PacketCarTelemetry.carTelemetryData
    pub isOpen: bool,                   // drs
    // PacketCarStatus.carStatusData
    pub isAllowed: bool,                // drsAllowed
}

impl fmt::Display for Drs
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if self.isOpen
        {
            write!(f, "[{}]", "DRS".black().on_green())
        }
        else if self.isAllowed
        {
            write!(f, "[{}]", "DRS".green())
        }
        else
        {
            write!(f, "[DRS]")
        }
    }
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
            VisualCompound::OldHard     | VisualCompound::Hard   => {
                write!(f, "{}{}{}", "(".white()  , self.visual, ")".white())
            },
            VisualCompound::OldMedium   | VisualCompound::Medium => {
                write!(f, "{}{}{}", "(".yellow() , self.visual, ")".yellow())
            },
            VisualCompound::OldSoft     | VisualCompound::Soft   => {
                write!(f, "{}{}{}", "(".red()    , self.visual, ")".red())
            },
            VisualCompound::OldSuperSoft                         => {
                write!(f, "{}{}{}", "(".magenta(), self.visual, ")".magenta())
            },
            VisualCompound::Inter                                => {
                write!(f, "{}{}{}", "(".green()  , self.visual, ")".green())
            },
            VisualCompound::OldWet      | VisualCompound::Wet    => {
                write!(f, "{}{}{}", "(".blue()   , self.visual, ")".blue())
            },
                                                               _ => {
                write!(f, "{}", self.actual)
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Telemetry
{
    // PacketCarTelemetry.carTelemetryData
    pub speed: Kph,                     // speed
    pub gear: Gear,                     // gear
    pub rpm: u16,                       // engineRPM
    pub leds: RevLights,                // revLightsBitValue
}

#[derive(Debug, Default, Clone)]
struct Times
{
    // PacketLap.laps
    pub sector1: Time,                  // sector1TimeInMS
    pub sector2: Time,                  // sector2TimeInMS
    pub sector3: Time,                  // sector2TimeInMS
    pub lastLap: Time,                  // lastLapTimeInMS
    pub current: Time,                  // currentLapTimeInMS
    pub possible: u32,
}

impl Times
{
    fn isBest(&mut self, period: Period, time: u32, id: u8, lap: u8) -> bool
    {
        // Formation Lap Fix.
        if time == 0
        {
            return false;
        }

        match period {
            Period::Sector1 => {
                if !self.sector1.isSet || self.sector1.inMS > time
                {
                    self.sector1.isSet = true;
                    self.sector1.byId = id;
                    self.sector1.onLap = lap;
                    self.sector1.inMS = time;
                    self.update_possible();
                    true
                }
                else
                {
                    false
                }
            },
            Period::Sector2 => {
                if !self.sector2.isSet || self.sector2.inMS > time
                {
                    self.sector2.isSet = true;
                    self.sector2.byId = id;
                    self.sector2.onLap = lap;
                    self.sector2.inMS = time;
                    self.update_possible();
                    true
                }
                else
                {
                    false
                }
            },
            Period::Sector3 => {
                if !self.sector3.isSet || self.sector3.inMS > time
                {
                    self.sector3.isSet = true;
                    self.sector3.byId = id;
                    self.sector3.onLap = lap;
                    self.sector3.inMS = time;
                    self.update_possible();
                    true
                }
                else
                {
                    false
                }
            },
            Period::LapTime => {
                if !self.lastLap.isSet || self.lastLap.inMS > time
                {
                    self.lastLap.isSet = true;
                    self.lastLap.byId = id;
                    self.lastLap.onLap = lap;
                    self.lastLap.inMS = time;
                    true
                }
                else
                {
                    false
                }
            },
        }
    }

    fn update_possible(&mut self) -> bool
    {
        if self.sector1.isSet && self.sector2.isSet && self.sector3.isSet
        {
            let newBestPossible: u32 = self.sector1.inMS as u32 + self.sector2.inMS as u32 + self.sector3.inMS as u32;

            if self.possible == 0 || self.possible > newBestPossible
            {
                self.possible = newBestPossible;
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Default, Clone)]
struct Car
{
    pub driver: Driver,
    pub team: Team,
    pub Drs: Drs,
    pub assist: Assists,
    pub tyres: Tyres,
    pub telemetry: Telemetry,
    pub time: Times,

    // PacketLap.laps
    pub spotGrid: u8,                   // gridPosition
    pub spotRace: u8,                   // carPosition
    pub lapNum: u8,                     // currentLapNum
    pub pitCount: u8,                   // numPitStops
    pub carStatus: CarState,            // driverStatus
    pub sector: u8,                     // sector
}

#[derive(Debug, Default, Clone, Copy)]
struct SessionLap {
    pub leader: u8,                     // MAX of PacketLap.laps.currentLapNum
    pub total: u8,                      // PacketSession.totalLaps
}

#[derive(Debug, Default, Clone, Copy)]
struct Time {
    pub inMS: u32,                      // MIN of PacketLap.laps.{lastLapTimeInMS}
    pub byId: u8,                       // Driver Index Number
    pub onLap: u8,                      // PacketLap.laps.currentLapNum
    pub isSet: bool,                    // Has this been set yet?
    pub isOB: bool,                     // Is Overall Best?
    pub isPB: bool,                     // Is Personal Best?
}

impl fmt::Display for Time
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let minutes = self.inMS / 60000;

        let time = if minutes > 0
        {
            let seconds =(self.inMS -(60000 * minutes)) / 1000;
            let milisec = self.inMS % 1000;
            format!("{}:{:02}.{:03}", minutes, seconds, milisec)
        }
        else
        {
            format!("{:.3}", self.inMS as f32 / 1000_f32)
        };

        write!(
            f,
            "{}",
            if self.isOB
            {
                format!("{:>8}", time.purple())
            }
            else if self.isPB
            {
                format!("{:>8}", time.green())
            }
            else if self.isSet
            {
                format!("{:>8}", time.yellow())
            }
            else
            {
                format!("{:>8}", time)
            }
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Best {
    pub sector1: Time,
    pub sector2: Time,
    pub sector3: Time,
    pub lapTime: Time,
    pub possible: u32,
}

#[derive(Debug)]
enum Period
{
    Sector1,
    Sector2,
    Sector3,
    LapTime,
}

impl Best
{
    fn isBest(&mut self, period: Period, time: u32, id: u8, lap: u8) -> bool
    {
        // Formation Lap Fix.
        if time == 0
        {
            return false;
        }

        match period {
            Period::Sector1 => {
                if !self.sector1.isSet || self.sector1.inMS > time
                {
                    self.sector1.isSet = true;
                    self.sector1.byId = id;
                    self.sector1.onLap = lap;
                    self.sector1.inMS = time;
                    self.update_possible();
                    true
                }
                else
                {
                    false
                }
            },
            Period::Sector2 => {
                if !self.sector2.isSet || self.sector2.inMS > time
                {
                    self.sector2.isSet = true;
                    self.sector2.byId = id;
                    self.sector2.onLap = lap;
                    self.sector2.inMS = time;
                    self.update_possible();
                    true
                }
                else
                {
                    false
                }
            },
            Period::Sector3 => {
                if !self.sector3.isSet || self.sector3.inMS > time
                {
                    self.sector3.isSet = true;
                    self.sector3.byId = id;
                    self.sector3.onLap = lap;
                    self.sector3.inMS = time;
                    self.update_possible();
                    true
                }
                else
                {
                    false
                }
            },
            Period::LapTime => {
                if !self.lapTime.isSet || self.lapTime.inMS > time
                {
                    self.lapTime.isSet = true;
                    self.lapTime.byId = id;
                    self.lapTime.onLap = lap;
                    self.lapTime.inMS = time;
                    true
                }
                else
                {
                    false
                }
            },
        }
    }

    fn update_possible(&mut self) -> bool
    {
        if self.sector1.isSet && self.sector2.isSet && self.sector3.isSet
        {
            let newBestPossible: u32 = self.sector1.inMS as u32 + self.sector2.inMS as u32 + self.sector3.inMS as u32;

            if self.possible == 0 || self.possible > newBestPossible
            {
                self.possible = newBestPossible;
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Default, Clone)]
struct Page
{
    participants: u8,                   // PacketParticipants.numActiveCars
    playerCarIndex: u8,                 // Always the last item, and so gives you the bounds of the array.
    positions: [usize; 22],
    r#type: Session,                    // PacketSession.sessionType
    ob: Best,
    car: [Car; 22],
    lap: SessionLap,
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:20777").expect("Couldn't bind to address.");
    println!("UDP Port Bound");

    let mut page = Page {
        positions: [usize::MAX; 22],
        ..Page::default()
    };

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

                    page.car[idx].driver.isAI               = p.participants[idx].aiControlled == 1;
                    page.car[idx].driver.id                 = p.participants[idx].driverId;
                    page.car[idx].driver.idNetwork          = p.participants[idx].networkId;
                    page.car[idx].team.id                   = p.participants[idx].teamId;
                    page.car[idx].team.isCustom             = p.participants[idx].myTeam == 1;
                    page.car[idx].driver.number             = p.participants[idx].raceNumber;
                    page.car[idx].driver.nationality        = p.participants[idx].nationality;
                    page.car[idx].driver.name               = p.participants[idx].name_to_string();
                    page.car[idx].driver.isTelemetryEnabled = p.participants[idx].yourTelemetry == 1;
                }
            }
            Packet::CarTelemetry(t) => {
                for i in 0..=page.playerCarIndex
                {
                    let idx = i as usize;

                    page.car[idx].Drs.isOpen                = t.carTelemetry[idx].drs == 1;
                    page.car[idx].telemetry.speed           = t.carTelemetry[idx].speed;
                    page.car[idx].telemetry.gear            = t.carTelemetry[idx].gear;
                    page.car[idx].telemetry.rpm             = t.carTelemetry[idx].engineRPM;
                    page.car[idx].telemetry.leds            = t.carTelemetry[idx].revLightsBitValue;
                }

            }
            Packet::CarStatus(s) => {
                for i in 0..=page.playerCarIndex
                {
                    let idx = i as usize;

                    page.car[idx].Drs.isAllowed             = s.carStatus[idx].drsAllowed == 1;
                    page.car[idx].assist.TC                 = s.carStatus[idx].tractionControl;
                    page.car[idx].assist.ABS                = s.carStatus[idx].antiLockBrakes;
                    page.car[idx].tyres.actual              = s.carStatus[idx].actualTyre;
                    page.car[idx].tyres.visual              = s.carStatus[idx].visualTyre;
                    page.car[idx].tyres.age                 = s.carStatus[idx].tyresAgeLaps;
                    page.car[idx].driver.underFlag          = s.carStatus[idx].vehicleFiaFlags;
                }
            }
            Packet::Lap(l) => {
                for i in 0..=page.playerCarIndex
                {
                    let idx = i as usize;
                    let lap = &l.laps[idx];
                    let car = &mut page.car[idx];
                    let pos = lap.carPosition as usize;

                    // Update car positions.
                    page.positions[pos] = idx;

                    // Skip empty slots.
                    if pos == 0
                    {
                        continue;
                    }

                    // Ignore Formation & First Lap
                    if lap.currentLapNum == 1 && lap.lapDistance < 0.0
                    {
                        continue;
                    }

                    // Update Leader Lap
                    if lap.currentLapNum > page.lap.leader
                    {
                        page.lap.leader = lap.currentLapNum;
                    }

                    match lap.sector
                    {
                        0 => {
                            // This is our first time in this sector.
                            if lap.sector != car.sector
                            {
                                let sector3 = lap.lastLapTimeInMS - (car.time.sector1.inMS + car.time.sector2.inMS);

                                car.time.sector3.inMS = sector3;
                                if page.ob.isBest(Period::Sector3, sector3, i, car.lapNum)
                                {
                                    car.time.sector3.isOB = true;
                                    car.time.sector3.isPB = true;
                                }
                                else if car.time.isBest(Period::Sector3, sector3, i, car.lapNum)
                                {
                                    car.time.sector3.isOB = false;
                                    car.time.sector3.isPB = true;
                                }
                                else
                                {
                                    car.time.sector3.isOB = false;
                                    car.time.sector3.isPB = false;
                                }

                                car.time.lastLap.inMS = lap.lastLapTimeInMS;
                                if page.ob.isBest(Period::LapTime, lap.lastLapTimeInMS, i, car.lapNum)
                                {
                                    car.time.lastLap.isOB = true;
                                    car.time.lastLap.isPB = true;
                                }
                                else if car.time.isBest(Period::LapTime, lap.lastLapTimeInMS, i, car.lapNum)
                                {
                                    car.time.lastLap.isPB = false;
                                    car.time.lastLap.isPB = true;
                                }
                                else
                                {
                                    car.time.lastLap.isPB = false;
                                    car.time.lastLap.isPB = false;
                                }
                            }

                            // Real Time Sector Time
                            car.time.sector1.inMS = lap.currentLapTimeInMS;
                        },
                        1 => {
                            // This is our first time in this sector.
                            if lap.sector != car.sector
                            {
                                car.time.sector1.inMS = lap.sector1TimeInMS as u32;
                                if page.ob.isBest(Period::Sector1, lap.sector1TimeInMS as u32, i, car.lapNum)
                                {
                                    car.time.sector1.isOB = true;
                                    car.time.sector1.isPB = true;
                                }
                                else if car.time.isBest(Period::Sector1, lap.sector1TimeInMS as u32, i, car.lapNum)
                                {
                                    car.time.sector1.isOB = false;
                                    car.time.sector1.isPB = true;
                                }
                                else
                                {
                                    car.time.sector1.isOB = false;
                                    car.time.sector1.isPB = false;
                                }
                            }

                            // Real Time Sector Time
                            car.time.sector2.inMS = lap.currentLapTimeInMS - car.time.sector1.inMS;
                        },
                        2 => {
                            // This is our first time in this sector.
                            if lap.sector != car.sector
                            {
                                car.time.sector2.inMS = lap.sector2TimeInMS as u32;
                                if page.ob.isBest(Period::Sector2, lap.sector2TimeInMS as u32, i, car.lapNum)
                                {
                                    car.time.sector2.isOB = true;
                                    car.time.sector2.isPB = true;
                                }
                                else if car.time.isBest(Period::Sector2, lap.sector2TimeInMS as u32, i, car.lapNum)
                                {
                                    car.time.sector2.isOB = false;
                                    car.time.sector2.isPB = true;
                                }
                                else
                                {
                                    car.time.sector2.isOB = false;
                                    car.time.sector2.isPB = false;
                                }
                            }

                            // Real Time Sector Time
                            car.time.sector3.inMS = lap.currentLapTimeInMS - (car.time.sector1.inMS + car.time.sector2.inMS);
                        },
                        _ => unreachable!()

                    }

                    // Now update the remaining new informaiton.
                    car.time.current.inMS = lap.currentLapTimeInMS;
                    car.time.lastLap.inMS = lap.lastLapTimeInMS;

                    car.spotGrid    = lap.gridPosition;
                    car.spotRace    = lap.carPosition;
                    car.lapNum      = lap.currentLapNum;
                    car.pitCount    = lap.numPitStops;
                    car.carStatus   = lap.driverStatus;
                    car.sector      = lap.sector;
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
                "{idx:2} {pos:2} {driver:>15} (##) {timeLastLap:>8} | {timeSector1:>8} {timeSector2:>8} {timeSector3:>8} | {timeCurrent:>8} | {lap:>3} {sector:^1} {tyre:>4} | {gear:>1} {DRS:^5} {speed:>3}",
                idx         = "ID",
                pos         = "P",
                driver      = "Driver",
                timeLastLap = "Last",
                timeSector1 = "S1",
                timeSector2 = "S2",
                timeSector3 = "S3",
                timeCurrent = "Time",
                lap         = "Lap",
                sector      = "S",
                tyre        = "Tyre",
                gear        = "G",
                DRS         = "DRS",
                speed       = "KPH",
            );

        // Cars
        // Need to figure out how to "zip" this together with:
        // let p: usize = page.positions[i as usize].into();
        // So that they are ordered correctly.
        for (pos, idx) in page.positions.iter().enumerate()
        {
            if pos == 0 || *idx > 22
            {   // Skip empty slots.
                continue
            }

            let car = &page.car[*idx];

            println!(
                "{idx:02} {pos:02} {driver} {timeLastLap:>8} | {timeSector1:>8} {timeSector2:>8} {timeSector3:>8} | {timeCurrent:>8} | {lap:>3} {sector:^1}  {tyre:>4} | {gear:>1} {DRS} {speed:>3}",
                driver      = car.driver.getDriver(),
                timeLastLap = car.time.lastLap,
                timeSector1 = car.time.sector1,
                timeSector2 = car.time.sector2,
                timeSector3 = car.time.sector3,
                timeCurrent = car.time.current,
                lap         = car.lapNum,
                sector      = car.sector,
                tyre        = car.tyres,
                gear        = car.telemetry.gear,
                DRS         = car.Drs,
                speed       = car.telemetry.speed,
            );
        }

        // Footer
        println!();

        // Bests
        println!(
            "{idx:2} {pos:2} {driver:>15}      {bestLapTime:>8} | {bestSector1:>8} {bestSector2:>8} {bestSector3:>8} | {bestPossible:>8.3}",
            idx         = "",
            pos         = "",
            driver      = "Bests",
            bestLapTime = page.ob.lapTime,
            bestSector1 = page.ob.sector1,
            bestSector2 = page.ob.sector2,
            bestSector3 = page.ob.sector3,
            bestPossible= page.ob.possible as f32 / 1000_f32,
        );

        // Footer
        println!();
    }
}
