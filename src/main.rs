#![allow(non_snake_case)]

use colored::*;
use std::fmt;
use std::net::UdpSocket;

static ESC: char = 27 as char;

mod packet;
use packet::*;

#[allow(dead_code)]
enum Packet {
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
    Unknown,
}

#[derive(Debug, Default, Clone)]
struct Driver {
    // From PacketParticipants.participants
    pub id: u8,                   // driverId
    pub idNetwork: u8,            // networkId
    pub number: u8,               // raceNumber
    pub nationality: u8,          // nationality
    pub isAI: bool,               // aiControlled
    pub isTelemetryEnabled: bool, // yourTelemetry
    pub name: String,             // name

    // PacketCarStatus.carStatusData
    pub underFlag: ZoneFlag, // vehicleFiaFlags
}

impl Driver {
    pub fn getDriver(&self) -> String {
        format!(
            "{:>15} ({:2})",
            match self.underFlag {
                ZoneFlag::Green => {
                    if self.isAI {
                        self.name.white().on_green()
                    } else {
                        self.name.yellow().on_green()
                    }
                }
                ZoneFlag::Blue => {
                    if self.isAI {
                        self.name.white().on_blue()
                    } else {
                        self.name.yellow().on_blue()
                    }
                }
                ZoneFlag::Yellow => {
                    if self.isAI {
                        self.name.white().on_yellow()
                    } else {
                        self.name.black().on_yellow()
                    }
                }
                _ => {
                    if self.isAI {
                        self.name.white()
                    } else {
                        self.name.yellow()
                    }
                }
            },
            self.number
        )
    }
}

#[derive(Debug, Default, Clone)]
struct Team {
    // From PacketParticipants.participants
    pub id: u8,         // teamId
    pub isCustom: bool, // myTeam
}

#[derive(Debug, Default, Clone)]
struct Drs {
    // PacketCarTelemetry.carTelemetryData
    pub isOpen: bool, // drs
    // PacketCarStatus.carStatusData
    pub isAllowed: bool, // drsAllowed
}

impl fmt::Display for Drs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.isOpen {
            write!(f, "[{}]", "DRS".black().on_green())
        } else if self.isAllowed {
            write!(f, "[{}]", "DRS".green())
        } else {
            write!(f, "[DRS]")
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Assists {
    // PacketCarStatus.carStatusData
    pub TC: TC,      // tractionControl
    pub ABS: Assist, // antiLockBrakes
}

#[derive(Debug, Default, Clone)]
struct Tyres {
    // PacketCarStatus.carStatusData
    pub actual: ActualCompound, // actualTyre
    pub visual: VisualCompound, // visualTyre
    pub age: u8,                // tyresAgeLaps
}

impl fmt::Display for Tyres {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.visual {
            VisualCompound::OldHard | VisualCompound::Hard => {
                write!(f, "{}{}{}", "(".white(), self.visual, ")".white())
            }
            VisualCompound::OldMedium | VisualCompound::Medium => {
                write!(f, "{}{}{}", "(".yellow(), self.visual, ")".yellow())
            }
            VisualCompound::OldSoft | VisualCompound::Soft => {
                write!(f, "{}{}{}", "(".red(), self.visual, ")".red())
            }
            VisualCompound::OldSuperSoft => {
                write!(f, "{}{}{}", "(".magenta(), self.visual, ")".magenta())
            }
            VisualCompound::Inter => {
                write!(f, "{}{}{}", "(".green(), self.visual, ")".green())
            }
            VisualCompound::OldWet | VisualCompound::Wet => {
                write!(f, "{}{}{}", "(".blue(), self.visual, ")".blue())
            }
            _ => {
                write!(f, "{}", self.actual)
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Telemetry {
    // PacketCarTelemetry.carTelemetryData
    pub speed: Kph,      // speed
    pub gear: Gear,      // gear
    pub rpm: u16,        // engineRPM
    pub leds: RevLights, // revLightsBitValue
}

#[derive(Debug, Default, Clone)]
struct Times {
    // PacketLap.laps
    pub sector1: Time, // sector1Time
    pub sector2: Time, // sector2Time
    pub sector3: Time, // sector2Time
    pub lastLap: Time, // lastLapTime
    pub current: Time, // currentLapTime
    pub interval: Time, // deltaToCarInFront
    pub leader: Time, // deltaToRaceLeader
    pub possible: u32,
}

impl Times {
    fn isBest(&mut self, period: Period, time: u32, idx: usize, lap: u8) -> bool {
        // Formation Lap Fix.
        if time == 0 {
            return false;
        }

        match period {
            Period::Sector1 => {
                if !self.sector1.isSet || self.sector1.inMS > time {
                    self.sector1.isSet = true;
                    self.sector1.byId = idx as u8;
                    self.sector1.onLap = lap;
                    self.sector1.inMS = time;
                    self.update_possible();
                    true
                } else {
                    false
                }
            }
            Period::Sector2 => {
                if !self.sector2.isSet || self.sector2.inMS > time {
                    self.sector2.isSet = true;
                    self.sector2.byId = idx as u8;
                    self.sector2.onLap = lap;
                    self.sector2.inMS = time;
                    self.update_possible();
                    true
                } else {
                    false
                }
            }
            Period::Sector3 => {
                if !self.sector3.isSet || self.sector3.inMS > time {
                    self.sector3.isSet = true;
                    self.sector3.byId = idx as u8;
                    self.sector3.onLap = lap;
                    self.sector3.inMS = time;
                    self.update_possible();
                    true
                } else {
                    false
                }
            }
            Period::LapTime => {
                if !self.lastLap.isSet || self.lastLap.inMS > time {
                    self.lastLap.isSet = true;
                    self.lastLap.byId = idx as u8;
                    self.lastLap.onLap = lap;
                    self.lastLap.inMS = time;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn update_possible(&mut self) -> bool {
        if self.sector1.isSet && self.sector2.isSet && self.sector3.isSet {
            let newBestPossible = self.sector1.inMS + self.sector2.inMS + self.sector3.inMS;

            if self.possible == 0 || self.possible > newBestPossible {
                self.possible = newBestPossible;
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Default, Clone)]
struct Car {
    pub driver: Driver,
    pub team: Team,
    pub Drs: Drs,
    pub assist: Assists,
    pub tyres: Tyres,
    pub telemetry: Telemetry,
    pub time: Times,

    // PacketLap.laps
    pub spotGrid: u8,        // gridPosition
    pub spotRace: u8,        // carPosition
    pub lapNum: u8,          // currentLapNum
    pub pitCount: u8,        // numPitStops
    pub carStatus: CarState, // driverStatus
    pub sector: u8,          // sector
}

#[derive(Debug, Default, Clone, Copy)]
struct SessionLap {
    pub leader: u8, // MAX of PacketLap.laps.currentLapNum
    pub total: u8,  // PacketSession.totalLaps
}

#[derive(Debug, Default, Clone, Copy)]
struct Time {
    pub inMS: u32,   // MIN of PacketLap.laps.{lastLapTimeInMS}
    pub byId: u8,    // Driver Index Number
    pub onLap: u8,   // PacketLap.laps.currentLapNum
    pub isSet: bool, // Has this been set yet?
    pub isOB: bool,  // Is Overall Best?
    pub isPB: bool,  // Is Personal Best?
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let minutes = self.inMS / 60000;

        let time = if minutes > 0 {
            let seconds = (self.inMS - (60000 * minutes)) / 1000;
            let milisec = self.inMS % 1000;
            format!("{}:{:02}.{:03}", minutes, seconds, milisec)
        } else {
            format!("{:.3}", self.inMS as f32 / 1000_f32)
        };

        write!(
            f,
            "{}",
            if self.isOB {
                format!("{:>8}", time.purple())
            } else if self.isPB {
                format!("{:>8}", time.green())
            } else if self.isSet {
                format!("{:>8}", time.yellow())
            } else {
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
enum Period {
    Sector1,
    Sector2,
    Sector3,
    LapTime,
}

impl Best {
    fn isBest(&mut self, period: Period, time: u32, idx: usize, lap: u8) -> bool {
        // Formation Lap Fix.
        if time == 0 {
            return false;
        }

        match period {
            Period::Sector1 => {
                if !self.sector1.isSet || self.sector1.inMS > time {
                    self.sector1.isSet = true;
                    self.sector1.byId = idx as u8;
                    self.sector1.onLap = lap;
                    self.sector1.inMS = time;
                    self.update_possible();
                    true
                } else {
                    false
                }
            }
            Period::Sector2 => {
                if !self.sector2.isSet || self.sector2.inMS > time {
                    self.sector2.isSet = true;
                    self.sector2.byId = idx as u8;
                    self.sector2.onLap = lap;
                    self.sector2.inMS = time;
                    self.update_possible();
                    true
                } else {
                    false
                }
            }
            Period::Sector3 => {
                if !self.sector3.isSet || self.sector3.inMS > time {
                    self.sector3.isSet = true;
                    self.sector3.byId = idx as u8;
                    self.sector3.onLap = lap;
                    self.sector3.inMS = time;
                    self.update_possible();
                    true
                } else {
                    false
                }
            }
            Period::LapTime => {
                if !self.lapTime.isSet || self.lapTime.inMS > time {
                    self.lapTime.isSet = true;
                    self.lapTime.byId = idx as u8;
                    self.lapTime.onLap = lap;
                    self.lapTime.inMS = time;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn update_possible(&mut self) -> bool {
        if self.sector1.isSet && self.sector2.isSet && self.sector3.isSet {
            let newBestPossible = self.sector1.inMS + self.sector2.inMS + self.sector3.inMS;

            if self.possible == 0 || self.possible > newBestPossible {
                self.possible = newBestPossible;
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Default, Clone)]
struct Page {
    participants: u8,   // PacketParticipants.numActiveCars
    playerCarIndex: u8, // Always the last item, and so gives you the bounds of the array.
    positions: [usize; 23],
    session: Session, // PacketSession.sessionType
    ob: Best,
    car: [Car; 23],
    lap: SessionLap,
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:20777").expect("Couldn't bind to address.");
    println!("UDP Port Bound");

    let mut page = Page {
        positions: [usize::MAX; 23],
        ..Page::default()
    };

    let mut buffer = [0; 1500];
    loop {
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
            PacketId::TyreSets => {
                // Smile and Wave Boys, Smile and Wave.
                Packet::Unknown
            },
            PacketId::MotionEx => {
                // Smile and Wave Boys, Smile and Wave.
                Packet::Unknown
            },
            PacketId::TimeTrial => {
                // Smile and Wave Boys, Smile and Wave.
                Packet::Unknown
            },
            PacketId::LapPositions => {
                // Smile and Wave Boys, Smile and Wave.
                Packet::Unknown
            },
            PacketId::Poisoned => {
                dbg!(header);
                println!(
                    "{}, of {size}, & of ID {:#?}",
                    "Unknown PacketId".red(),
                    header.packetId
                );
                Packet::Unknown
            }
        };

        match packet {
            Packet::Event(e) => {
                match e.eventType {
                    EventType::SessionStarted => {
                        // We have a new sessions, so let's reset everything back to defualt.
                        page = Page {
                            positions: [usize::MAX; 23],
                            ..Page::default()
                        };
                    },
                    _ => {
                        // Not handling these.
                    }
                }
            }
            Packet::Session(s) => {
                page.playerCarIndex = s.header.playerCarIndex;
                page.lap.total = s.totalLaps;
                page.session = s.sessionType;
            }
            Packet::Participants(p) => {
                page.participants = p.numActiveCars;

                for i in 0..=page.playerCarIndex {
                    let idx = i as usize;

                    page.car[idx].driver.isAI = p.participants[idx].aiControlled == 1;
                    page.car[idx].driver.id = p.participants[idx].driverId;
                    page.car[idx].driver.idNetwork = p.participants[idx].networkId;
                    page.car[idx].team.id = p.participants[idx].teamId;
                    page.car[idx].team.isCustom = p.participants[idx].myTeam == 1;
                    page.car[idx].driver.number = p.participants[idx].raceNumber;
                    page.car[idx].driver.nationality = p.participants[idx].nationality;
                    page.car[idx].driver.name = p.participants[idx].name_to_string();
                    page.car[idx].driver.isTelemetryEnabled =
                        p.participants[idx].yourTelemetry == 1;
                }
            }
            Packet::CarTelemetry(t) => {
                for i in 0..=page.playerCarIndex {
                    let idx = i as usize;

                    page.car[idx].Drs.isOpen = t.carTelemetry[idx].drs == 1;
                    page.car[idx].telemetry.speed = t.carTelemetry[idx].speed;
                    page.car[idx].telemetry.gear = t.carTelemetry[idx].gear;
                    page.car[idx].telemetry.rpm = t.carTelemetry[idx].engineRPM;
                    page.car[idx].telemetry.leds = t.carTelemetry[idx].revLightsBitValue;
                }
            }
            Packet::CarStatus(s) => {
                for i in 0..=page.playerCarIndex {
                    let idx = i as usize;

                    page.car[idx].Drs.isAllowed = s.carStatus[idx].drsAllowed == 1;
                    page.car[idx].assist.TC = s.carStatus[idx].tractionControl;
                    page.car[idx].assist.ABS = s.carStatus[idx].antiLockBrakes;
                    page.car[idx].tyres.actual = s.carStatus[idx].actualTyre;
                    page.car[idx].tyres.visual = s.carStatus[idx].visualTyre;
                    page.car[idx].tyres.age = s.carStatus[idx].tyresAgeLaps;
                    page.car[idx].driver.underFlag = s.carStatus[idx].vehicleFiaFlags;
                }
            }
            Packet::Lap(l) => {
                for (idx, car) in l.cars.iter().enumerate()
                {
                    let pcs = &mut page.car[idx];
                    let pos = car.racePosition as usize;

                    if pos == 0 {
                        continue;
                    }

                    // Update car positions.
                    page.positions[pos] = idx;

                    // Ignore Formation & First Lap
                    if car.lapDistance < 0.0 {
                        continue;
                    }

                    // Update Leader Lap
                    if car.currentLapNum > page.lap.leader {
                        page.lap.leader = car.currentLapNum;
                    }

                    // Interval
                    pcs.time.interval.inMS =
                        car.deltaToCarInFrontMinutesPart as u32 * 60 * 1000 +
                        car.deltaToCarInFrontMSPart as u32;

                    // Leader
                    pcs.time.leader.inMS =
                        car.deltaToRaceLeaderMinutesPart as u32 * 60 * 1000 +
                        car.deltaToRaceLeaderMSPart as u32;

                    match car.sector {
                        0 => {  // Sector 1 (New Lap)
                            // This is our first time in this sector?
                            if car.sector != pcs.sector {
                                // Clear any stats from the previous lap.
                                pcs.time.sector1.isOB = false;
                                pcs.time.sector1.isPB = false;
                                pcs.time.lastLap.isOB = false;
                                pcs.time.lastLap.isPB = false;

                                // Calculate Sector 3's Split Time.
                                let sector3 = car.lastLapTimeInMS - (pcs.time.sector1.inMS + pcs.time.sector2.inMS);
                                pcs.time.sector3.inMS = sector3;

                                // Check to see if it's a Personal and / or Overall Best.
                                if page.ob.isBest(Period::Sector3, sector3, idx, pcs.lapNum - 1) {
                                    pcs.time.sector3.isOB = true;
                                    pcs.time.sector3.isPB = true;
                                } else if pcs.time.isBest(Period::Sector3, sector3, idx, pcs.lapNum - 1) {
                                    pcs.time.sector3.isOB = false;
                                    pcs.time.sector3.isPB = true;
                                } else {
                                    pcs.time.sector3.isOB = false;
                                    pcs.time.sector3.isPB = false;
                                }

                                // And check the overall lap time.
                                pcs.time.lastLap.inMS = car.lastLapTimeInMS;
                                if page.ob.isBest(
                                    Period::LapTime,
                                    car.lastLapTimeInMS,
                                    idx,
                                    pcs.lapNum - 1,
                                ) {
                                    pcs.time.lastLap.isOB = true;
                                    pcs.time.lastLap.isPB = true;
                                } else if pcs.time.isBest(
                                    Period::LapTime,
                                    car.lastLapTimeInMS,
                                    idx,
                                    pcs.lapNum - 1,
                                ) {
                                    pcs.time.lastLap.isPB = false;
                                    pcs.time.lastLap.isPB = true;
                                } else {
                                    pcs.time.lastLap.isPB = false;
                                    pcs.time.lastLap.isPB = false;
                                }
                            }

                            // Live Updateing Sector Time
                            pcs.time.sector1.inMS = car.currentLapTimeInMS;
                        }
                        1 => {  // Sector 2
                            // This is our first time in this sector.
                            if car.sector != pcs.sector {
                                // Clear any stats from the previous lap.
                                pcs.time.sector2.isOB = false;
                                pcs.time.sector2.isPB = false;

                                // Calculate Sector 1's Split Time.
                                let sectorTimeInMS: u32 =
                                    car.sector1TimeMinutesPart as u32 * 60 * 1000 +
                                    car.sector1TimeMSPart as u32 ;

                                pcs.time.sector1.inMS = sectorTimeInMS;
                                if page.ob.isBest(
                                    Period::Sector1,
                                    sectorTimeInMS,
                                    idx,
                                    pcs.lapNum,
                                ) {
                                    pcs.time.sector1.isOB = true;
                                    pcs.time.sector1.isPB = true;
                                } else if pcs.time.isBest(
                                    Period::Sector1,
                                    sectorTimeInMS,
                                    idx,
                                    pcs.lapNum,
                                ) {
                                    pcs.time.sector1.isOB = false;
                                    pcs.time.sector1.isPB = true;
                                } else {
                                    pcs.time.sector1.isOB = false;
                                    pcs.time.sector1.isPB = false;
                                }
                            }

                            // Live Updateing Sector Time
                            if pcs.time.sector1.inMS < car.currentLapTimeInMS {
                                pcs.time.sector2.inMS = car.currentLapTimeInMS - pcs.time.sector1.inMS;
                            }
                        }
                        2 => {  // Sector 3
                            // This is our first time in this sector.
                            if car.sector != pcs.sector {
                                // Clear any stats from the previous lap.
                                pcs.time.sector2.isOB = false;
                                pcs.time.sector2.isPB = false;

                                // Calculate Sector 2's Split Time.
                                let sectorTimeInMS: u32 =
                                    car.sector2TimeMinutesPart as u32 * 60 * 1000 +
                                    car.sector2TimeMSPart as u32 ;

                                pcs.time.sector2.inMS = sectorTimeInMS;
                                if page.ob.isBest(
                                    Period::Sector2,
                                    sectorTimeInMS,
                                    idx,
                                    pcs.lapNum,
                                ) {
                                    pcs.time.sector2.isOB = true;
                                    pcs.time.sector2.isPB = true;
                                } else if pcs.time.isBest(
                                    Period::Sector2,
                                    sectorTimeInMS,
                                    idx,
                                    pcs.lapNum,
                                ) {
                                    pcs.time.sector2.isOB = false;
                                    pcs.time.sector2.isPB = true;
                                } else {
                                    pcs.time.sector2.isOB = false;
                                    pcs.time.sector2.isPB = false;
                                }
                            }

                            // Live Updates
                            if (pcs.time.sector1.inMS + pcs.time.sector2.inMS) < car.currentLapTimeInMS {
                                pcs.time.sector3.inMS = car.currentLapTimeInMS - (pcs.time.sector1.inMS + pcs.time.sector2.inMS);
                            }
                        }
                        _ => unreachable!(),
                    }

                    // Now update the remaining new informaiton.
                    pcs.time.current.inMS = car.currentLapTimeInMS;
                    pcs.time.lastLap.inMS = car.lastLapTimeInMS;

                    pcs.spotGrid = car.gridPosition;
                    pcs.spotRace = car.racePosition;
                    pcs.lapNum = car.currentLapNum;
                    pcs.pitCount = car.numPitStops;
                    pcs.carStatus = car.driverStatus;
                    pcs.sector = car.sector;
                }
            }
            _ => {
                continue;
            }
        }

        // Clear Screen & Corsor @ Top Left
        print!("{ESC}c");

        println!(
            "{session:>5} {lapLeader:02} {lapTotal:02}",
            session   = page.session,
            lapLeader = page.lap.leader,
            lapTotal  = page.lap.total,
        );

        // Header
        println!(
                "{pos:2} {driver:>15} (##) {timeLastLap:>8} | {interval:>8} | {leader:>8} | {timeSector1:>8} {timeSector2:>8} {timeSector3:>8} | {timeCurrent:>8} | {lap:>3} {sector:^1} {tyre:>4} | {gear:>1} {DRS:^5} {speed:>3} | {state:^5}",
                pos         = "P",
                driver      = "Driver",
                timeLastLap = "Last",
                interval    = "Interval",
                leader      = "Leader",
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
                state       = "State"
            );

        for (pos, idx) in page.positions.iter().enumerate() {
            if *idx > page.playerCarIndex as usize {
                // Skip empty slots.
                continue;
            }

            let car = &page.car[*idx];

            println!(
                "{pos:02} {driver} {timeLastLap:>8} | {interval:>8} | {leader:>8} | {timeSector1:>8} {timeSector2:>8} {timeSector3:>8} | {timeCurrent:>8} | {lap:>3} {sector:^1}  {tyre:>4} | {gear:>1} {DRS} {speed:>3} | {state:^5}",
                driver      = car.driver.getDriver(),
                timeLastLap = car.time.lastLap,
                interval    = car.time.interval,
                leader      = car.time.leader,
                timeSector1 = car.time.sector1,
                timeSector2 = car.time.sector2,
                timeSector3 = car.time.sector3,
                timeCurrent = car.time.current,
                lap         = car.lapNum,
                sector      = car.sector,
                tyre        = car.tyres,
                gear        = car.telemetry.gear,
                DRS         = car.Drs,
                speed       = car.telemetry.speed.kph,
                state       = car.carStatus
            );
        }

        // Header
        println!();

        // Bests
        println!(
            "{pos:2} {driver:>15}      {bestLapTime:>8} | {interval:>8} | {leader:>8} | {bestSector1:>8} {bestSector2:>8} {bestSector3:>8} | {bestPossible:>8.3}",
            pos         = "",
            driver      = "Bests",
            interval    = "",
            leader      = "",
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
