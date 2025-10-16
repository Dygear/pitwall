#![allow(non_snake_case)]
#![allow(dead_code)]
use colored::Colorize;
use std::mem::size_of;
use std::fmt;

// https://answers.ea.com/t5/General-Discussion/F1-22-UDP-Specification/td-p/11551274
// https://forums.ea.com/blog/f1-games-game-info-hub-en/ea-sports%e2%84%a2-f1%c2%ae25-udp-specification/12187347

//
// # Packet Header
// Each packet has the following header:
// Size: 29 Bytes
//

#[repr(C, packed)] // Size: 29 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct Header
{
    pub packetFormat: u16,              // 2025
    pub gameYear: u8,                   // Game year - last two digits e.g. 25
    pub gameMajorVersion: u8,           // Game major version - "X.00"
    pub gameMinorVersion: u8,           // Game minor version - "1.XX"
    pub packetVersion: u8,              // Version of this packet type, all start from 1
    pub packetId: PacketId,             // u8 - Identifier for the packet type, see below
    pub sessionUID: u64,                // Unique identifier for the session
    pub sessionTime: f32,               // Session timestamp
    pub frameIdentifier: u32,           // Identifier for the frame the data was retrieved on
    pub overallFrameIdentifier: u32,    // Overall identifier for the frame the data was retrieved on, doesn't go back after flashbacks
    pub playerCarIndex: u8,             // Index of player's car in the array
    pub secondaryPlayerCarIndex: u8,    // Index of secondary player's car in the array (splitscreen) 255 if no second player
}

impl Header
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            packetFormat            : u16::from_le_bytes([bytes[0], bytes[1]]),
            gameYear                : bytes[2],
            gameMajorVersion        : bytes[3],
            gameMinorVersion        : bytes[4],
            packetVersion           : bytes[5],
            packetId                : PacketId::from_u8(bytes[6]),
            sessionUID              : u64::from_le_bytes([bytes[ 7], bytes[ 8], bytes[ 9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14]]),
            sessionTime             : f32::from_le_bytes([bytes[15], bytes[16], bytes[17], bytes[18]]),
            frameIdentifier         : u32::from_le_bytes([bytes[19], bytes[20], bytes[21], bytes[22]]),
            overallFrameIdentifier  : u32::from_le_bytes([bytes[23], bytes[24], bytes[25], bytes[26]]),
            playerCarIndex          : bytes[27],
            secondaryPlayerCarIndex : bytes[28],
        }
    }

    pub fn get_version(&self) -> String
    {
        format!(
            "{}.{:0>2}",
            self.gameMajorVersion,
            self.gameMinorVersion
        )
    }
}

/// # Packet IDs
/// The packets IDs are as follows
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum PacketId {
    Motion = 0,                         // Contains all motion data for player’s car – only sent while player is in control
    Session = 1,                        // Data about the session – track, time left
    Lap = 2,                            // Data about all the lap times of cars in the session
    Event = 3,                          // Various notable events that happen during a session
    Participants = 4,                   // List of participants in the session, mostly relevant for multiplayer
    CarSetups = 5,                      // Packet detailing car setups for cars in the race
    CarTelemetry = 6,                   // Telemetry data for all cars
    CarStatus = 7,                      // Status data for all cars
    FinalClassification = 8,            // Final classification confirmation at the end of a race
    LobbyInfo = 9,                      // Information about players in a multiplayer lobby
    CarDamage = 10,                     // Damage status for all cars
    SessionHistory = 11,                // Lap and tyre data for session
    TyreSets = 12,                      // Extended tyre set data
    MotionEx = 13,                      // Extended motion data for player car
    TimeTrial = 14,                     // Time Trial specific data
    LapPositions = 15,                  // Lap positions on each lap so a chart can be constructed
    #[default]
    Poisoned = 255,
}

impl PacketId {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => PacketId::Motion,
            1 => PacketId::Session,
            2 => PacketId::Lap,
            3 => PacketId::Event,
            4 => PacketId::Participants,
            5 => PacketId::CarSetups,
            6 => PacketId::CarTelemetry,
            7 => PacketId::CarStatus,
            8 => PacketId::FinalClassification,
            9 => PacketId::LobbyInfo,
            10=> PacketId::CarDamage,
            11=> PacketId::SessionHistory,
            12=> PacketId::TyreSets,
            13=> PacketId::MotionEx,
            14=> PacketId::TimeTrial,
            15=> PacketId::LapPositions,
            _ => PacketId::Poisoned,
        }
    }
}

/// # Motion Packet
/// The motion packet gives physics data for all the cars being driven. There is additional data for the car being driven with the goal of being able to drive a motion platform setup.
/// N.B. For the normalised vectors below, to convert to float values divide by 32767.0f – 16-bit signed values are used to pack the data and on the assumption that direction values are always between -1.0f and 1.0f.
/// Frequency: Rate as specified in menus
/// Size: 1464 bytes
/// Version: 1
#[repr(C, packed)] // Size: 60 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct CarMotion
{
    pub worldPosition: Vector,          // World space position
    pub worldVelocity: Vector,          // Velocity in world space
    pub worldForward: Direction,        // World space forward direction (normalised)
    pub worldRight: Direction,          // World space right direction (normalised)
    pub gForce: Forces,                 // G-Forces
    pub yaw: f32,                       // Yaw angle in radians
    pub pitch: f32,                     // Pitch angle in radians
    pub roll: f32,                      // Roll angle in radians
}

impl CarMotion
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            worldPosition     :    Vector::unpack(&bytes[ 0..12]),
            worldVelocity     :    Vector::unpack(&bytes[12..24]),
            worldForward      : Direction::unpack(&bytes[24..30]),
            worldRight        : Direction::unpack(&bytes[30..36]),
            gForce            :    Forces::unpack(&bytes[36..48]),
            yaw               : f32::from_le_bytes([bytes[48], bytes[49], bytes[50], bytes[51]]),
            pitch             : f32::from_le_bytes([bytes[52], bytes[53], bytes[54], bytes[55]]),
            roll              : f32::from_le_bytes([bytes[56], bytes[57], bytes[58], bytes[59]]),
        }
    }
}

#[repr(C, packed)] // Size: 16 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct Vector
{
    pub X: f32,
    pub Y: f32,
    pub Z: f32,
}

impl Vector
{
    fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            X: f32::from_le_bytes([bytes[ 0], bytes[ 1], bytes[ 2], bytes[ 3]]),
            Y: f32::from_le_bytes([bytes[ 4], bytes[ 5], bytes[ 6], bytes[ 7]]),
            Z: f32::from_le_bytes([bytes[ 8], bytes[ 9], bytes[10], bytes[11]]),
        }
    }
}

#[repr(C, packed)] // Size: 6 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct Direction
{
    pub X: i16,
    pub Y: i16,
    pub Z: i16,
}

impl Direction
{
    fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            X: i16::from_le_bytes([bytes[ 0], bytes[ 1]]),
            Y: i16::from_le_bytes([bytes[ 2], bytes[ 3]]),
            Z: i16::from_le_bytes([bytes[ 4], bytes[ 5]]),
        }
    }
}

#[repr(C, packed)] // Size: 16 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct Forces
{
    pub Lateral     : f32,
    pub Longitudinal: f32,
    pub Vertical    : f32,
}

impl Forces
{
    fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            Lateral     : f32::from_le_bytes([bytes[ 0], bytes[ 1], bytes[ 2], bytes[ 3]]),
            Longitudinal: f32::from_le_bytes([bytes[ 4], bytes[ 5], bytes[ 6], bytes[ 7]]),
            Vertical    : f32::from_le_bytes([bytes[ 8], bytes[ 9], bytes[10], bytes[11]]),
        }
    }
}

#[repr(C, packed)] // Size: 16 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct Wheels
{
    pub RL: f32,
    pub RR: f32,
    pub FL: f32,
    pub FR: f32,
}

impl Wheels
{
    fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            RL: f32::from_le_bytes([bytes[ 0], bytes[ 1], bytes[ 2], bytes[ 3]]),
            RR: f32::from_le_bytes([bytes[ 4], bytes[ 5], bytes[ 6], bytes[ 7]]),
            FL: f32::from_le_bytes([bytes[ 8], bytes[ 9], bytes[10], bytes[11]]),
            FR: f32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]])
        }
    }
}

#[repr(C, packed)] // Size: 1464 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct PacketMotion
{
    pub header: Header,                 // 24 Bytes - Header

    pub carMotion: [CarMotion; 22],     // 60 Bytes * 22 - Data for all cars on track

    // Extra player car ONLY data
    pub suspensionPosition: Wheels,     // Note: All wheel arrays have the following order:
    pub suspensionVelocity: Wheels,     // RL, RR, FL, FR
    pub suspensionAcceleration: Wheels, // RL, RR, FL, FR
    pub wheelSpeed: Wheels,             // Speed of each wheel
    pub wheelSlip: Wheels,              // Slip ratio for each wheel
    pub localVelocity: Vector,          // Velocity in local space
    pub angularVelocity: Vector,        // Angular velocity
    pub angularAcceleration: Vector,    // Angular acceleration
    pub frontWheelsAngle: f32,          // Current front wheels angle in radians
}

// Size: 1464 Bytes
impl PacketMotion
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            header: Header::unpack(bytes),

            carMotion: Self::carMotion(&bytes[size_of::<Header>()..size_of::<Header>()+(size_of::<CarMotion>()*22)]),

            // Extra player car ONLY data
            suspensionPosition    : Wheels::unpack(&bytes[1349..1365]),
            suspensionVelocity    : Wheels::unpack(&bytes[1365..1381]),
            suspensionAcceleration: Wheels::unpack(&bytes[1381..1397]),
            wheelSpeed            : Wheels::unpack(&bytes[1397..1413]),
            wheelSlip             : Wheels::unpack(&bytes[1413..1429]),
            localVelocity         : Vector::unpack(&bytes[1429..1441]),
            angularVelocity       : Vector::unpack(&bytes[1441..1453]),
            angularAcceleration   : Vector::unpack(&bytes[1453..1465]),
            frontWheelsAngle      : f32::from_le_bytes([bytes[1465], bytes[1466], bytes[1467], bytes[1468]]),
        }
    }

    pub fn carMotion(bytes: &[u8]) -> [CarMotion; 22]
    {
        let mut cm = [CarMotion::default(); 22];
        let size = size_of::<CarMotion>();

        for (i, cm) in cm.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *cm = CarMotion::unpack(&bytes[s..e]);
        }

        cm
    }
}


/**
 * # Session Packet
 * The session packet includes details about the current session in progress.
 * Frequency: 2 per second
 * Size: 632 bytes
 * Version: 1
 */
#[repr(C, packed)] // Size: 5 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct MarshalZone
{
    pub zoneStart: f32,                 // Fraction (0..1) of way through the lap the marshal zone starts
    pub zoneFlag: ZoneFlag,             // i8
}

impl MarshalZone
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            zoneStart: f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            zoneFlag: ZoneFlag::from_u8_to_i8(&bytes[4]),
        }
    }
}

#[repr(i8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum ZoneFlag {
    Invalid = -1,
    None = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
    Red = 4,
    #[default]
    Unknown = 127
}

impl ZoneFlag
{
    pub fn from_u8_to_i8(byte: &u8) -> Self
    {
        match *byte as i8
        {
           -1 => ZoneFlag::Invalid,
            0 => ZoneFlag::None,
            1 => ZoneFlag::Green,
            2 => ZoneFlag::Blue,
            3 => ZoneFlag::Yellow,
            4 => ZoneFlag::Red,
            _ => ZoneFlag::Unknown,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Session {
    Unknown = 0,
    Practice1 = 1,
    Practice2 = 2,
    Practice3 = 3,
    ShortPractice = 4,
    Quali1 = 5,
    Quali2 = 6,
    Quali3 = 7,
    ShortQuli = 8,
    OneShotQuli = 9,
    Race = 10,
    Race2 = 11,
    Race3 = 12,
    TimeTrial = 13,
    #[default]
    Poisoned = 255,
}

impl Session
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => Session::Unknown,
            1 => Session::Practice1,
            2 => Session::Practice2,
            3 => Session::Practice3,
            4 => Session::ShortPractice,
            5 => Session::Quali1,
            6 => Session::Quali2,
            7 => Session::Quali3,
            8 => Session::ShortQuli,
            9 => Session::OneShotQuli,
            10=> Session::Race,
            11=> Session::Race2,
            12=> Session::Race3,
            13=> Session::TimeTrial,
            _ => Session::Poisoned,
        }
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self
        {
            Self::Practice1     => write!(f, "FP1"),
            Self::Practice2     => write!(f, "FP2"),
            Self::Practice3     => write!(f, "FP3"),
            Self::ShortPractice => write!(f, "SP"),
            Self::Quali1        => write!(f, "Q1"),
            Self::Quali2        => write!(f, "Q2"),
            Self::Quali3        => write!(f, "Q3"),
            Self::ShortQuli     => write!(f, "SQ"),
            Self::OneShotQuli   => write!(f, "1SQ"),
            Self::Race          => write!(f, "Race"),
            Self::Race2         => write!(f, "Race2"),
            Self::Race3         => write!(f, "Race3"),
            Self::TimeTrial     => write!(f, "Time"),
            _                   => write!(f, ""),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Weather {
    Clear = 0,
    LightCloud = 1,
    Overcast = 2,
    RainLight = 3,
    RainHeavy = 4,
    RainStorm = 5,
    #[default]
    Poisoned = 255
}

impl Weather
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => Weather::Clear,
            1 => Weather::LightCloud,
            2 => Weather::Overcast,
            3 => Weather::RainLight,
            4 => Weather::RainHeavy,
            5 => Weather::RainStorm,
            _ => Weather::Poisoned,
        }
    }
}

#[repr(i8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Temperature {
    Up = 0,
    Down = 1,
    None = 2,
    #[default]
    Poisoned = 127
}

impl Temperature
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => Temperature::Up,
            1 => Temperature::Down,
            2 => Temperature::None,
            _ => Temperature::Poisoned,
        }
    }
}

#[repr(C, packed)] // Size: 8 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct WeatherForecast
{
    pub sessionType: Session,           // u8
    pub timeOffset: u8,                 // Time in minutes the forecast is for
    pub weather: Weather,               // u8
    pub trackTemperature: i8,           // Track temp. in degrees Celsius
    pub trackChange: Temperature,       // i8
    pub airTemperature: i8,             // Air temp. in degrees celsius
    pub airChange: Temperature,         // i8
    pub rainPercentage: u8,             // Rain percentage (0-100)
}

impl WeatherForecast
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            sessionType     : Session::from_u8(&bytes[0]),
            timeOffset      : bytes[1],
            weather         : Weather::from_u8(&bytes[2]),
            trackTemperature: bytes[3] as i8,
            trackChange     : Temperature::from_u8(&bytes[4]),
            airTemperature  : bytes[5] as i8,
            airChange       : Temperature::from_u8(&bytes[6]),
            rainPercentage  : bytes[7],
        }
    }
}

#[repr(C, packed)] // Size: 753 Bytes
#[derive(Debug, Clone, Copy)]
pub struct PacketSession
{
    pub header: Header,                         // 29 Bytes - Header

    pub weather: Weather,                       // u8
    pub trackTemperature: i8,                   // Track temp. in degrees celsius
    pub airTemperature: i8,                     // Air temp. in degrees celsius
    pub totalLaps: u8,                          // Total number of laps in this race
    pub trackLength: u16,                       // Track length in metres
    pub sessionType: Session,                   // u8
    pub trackId: i8,                            // -1 for unknown, see appendix
    pub formula: Formula,                       // u8
    pub sessionTimeLeft: u16,                   // Time left in session in seconds
    pub sessionDuration: u16,                   // Session duration in seconds
    pub pitSpeedLimit: u8,                      // Pit speed limit in kilometres per hour
    pub gamePaused: u8,                         // Whether the game is paused – network game only
    pub isSpectating: u8,                       // Whether the player is spectating
    pub spectatorCarIndex: u8,                  // Index of the car being spectated
    pub sliProNativeSupport: SLIPro,            // u8
    pub numMarshalZones: u8,                    // Number of marshal zones to follow
    pub marshalZones: [MarshalZone; 21],        // 105 Bytes - List of marshal zones – max 21
    pub safetyCarStatus: SafetyCar,             // u8
    pub networkGame: NetworkGame,               // u8
    pub numWeatherForecasts: u8,                // Number of weather samples to follow
    pub weatherForecast: [WeatherForecast; 64], // 512 Bytes - of weather forecast samples
    pub forecastAccuracy: Accuracy,             // u8
    pub aiDifficulty: u8,                       // AI Difficulty rating – 0-110
    pub seasonLinkIdentifier: u32,              // Identifier for season - persists across saves
    pub weekendLinkIdentifier: u32,             // Identifier for weekend - persists across saves
    pub sessionLinkIdentifier: u32,             // Identifier for session - persists across saves
    pub pitStopWindowIdealLap: u8,              // Ideal lap to pit on for current strategy (player)
    pub pitStopWindowLatestLap: u8,             // Latest lap to pit on for current strategy (player)
    pub pitStopRejoinPosition: u8,              // Predicted position to rejoin at (player)
    pub steeringAssist: Assist,                 // u8
    pub brakingAssist: u8,                      // 0 = off, 1 = low, 2 = medium, 3 = high
    pub gearboxAssist: u8,                      // 1 = manual, 2 = manual & suggested gear, 3 = auto
    pub pitAssist: Assist,                      // u8
    pub pitReleaseAssist: Assist,               // u8
    pub ERSAssist: Assist,                      // u8
    pub DRSAssist: Assist,                      // u8
    pub dynamicRacingLine: u8,                  // 0 = off, 1 = corners only, 2 = full
    pub dynamicRacingLineType: u8,              // 0 = 2D, 1 = 3D
    pub gameMode: u8,                           // Game mode id - see appendix
    pub ruleSet: u8,                            // Ruleset - see appendix
    pub timeOfDay: u32,                         // Local time of day - minutes since midnight
    pub sessionLength: SessionLength,           // u8
    pub speedUnitsLeadPlayer: u8,               // 0 = MPH, 1 = KPH
    pub temperatureUnitsLeadPlayer: u8,         // 0 = Celsius, 1 = Fahrenheit
    pub speedUnitsSecondaryPlayer: u8,          // 0 = MPH, 1 = KPH
    pub temperatureUnitsSecondaryPlayer: u8,    // 0 = Celsius, 1 = Fahrenheit
    pub numSafetyCarPeriods: u8,                // Number of safety cars called during session
    pub numVirtualSafetyCarPeriods: u8,         // Number of virtual safety cars called during session
    pub numRedFlagPeriods: u8,                  // Number of red flags called during session
    pub equalCarPerformance: u8,                // 0 = Off, 1 = On
    pub recoveryMode: u8,                       // 0 = None, 1 = Flashbacks, 2 = Auto-recovery
    pub flashbackLimit: u8,                     // 0 = Low, 1 = Medium, 2 = High, 3 = Unlimited
    pub surfaceType: u8,                        // 0 = Simplified, 1 = Realistic
    pub lowFuelMode: u8,                        // 0 = Easy, 1 = Hard
    pub raceStarts: u8,                         // 0 = Manual, 1 = Assisted
    pub tyreTemperature: u8,                    // 0 = Surface only, 1 = Surface & Carcass
    pub pitLaneTyreSim: u8,                     // 0 = On, 1 = Off
    pub carDamage: u8,                          // 0 = Off, 1 = Reduced, 2 = Standard, 3 = Simulation
    pub carDamageRate: u8,                      // 0 = Reduced, 1 = Standard, 2 = Simulation
    pub collisions: u8,                         // 0 = Off, 1 = Player-to-Player Off, 2 = On
    pub collisionsOffForFirstLapOnly: u8,       // 0 = Disabled, 1 = Enabled
    pub mpUnsafePitRelease: u8,                 // 0 = On, 1 = Off (Multiplayer)
    pub mpOffForGriefing: u8,                   // 0 = Disabled, 1 = Enabled (Multiplayer)
    pub cornerCuttingStringency: u8,            // 0 = Regular, 1 = Strict
    pub parcFermeRules: u8,                     // 0 = Off, 1 = On
    pub pitStopExperience: u8,                  // 0 = Automatic, 1 = Broadcast, 2 = Immersive
    pub safetyCar: u8,                          // 0 = Off, 1 = Reduced, 2 = Standard, 3 = Increased
    pub safetyCarExperience: u8,                // 0 = Broadcast, 1 = Immersive
    pub formationLap: u8,                       // 0 = Off, 1 = On
    pub formationLapExperience: u8,             // 0 = Broadcast, 1 = Immersive
    pub redFlags: u8,                           // 0 = Off, 1 = Reduced, 2 = Standard, 3 = Increased
    pub affectsLicenceLevelSolo: u8,            // 0 = Off, 1 = On
    pub affectsLicenceLevelMP: u8,              // 0 = Off, 1 = On
    pub numSessionsInWeekend: u8,               // Number of session in following array
    pub weekendStructure: [u8; 12],             // List of session types to show weekend structure - see appendix for types
    pub sector2LapDistanceStart: f32,           // Distance in m around track where sector 2 starts
    pub sector3LapDistanceStart: f32,           // Distance in m around track where sector 3 starts
}

impl PacketSession
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            header                          : Header::unpack(bytes),

            weather                         : Weather::from_u8(&bytes[29]),
            trackTemperature                : bytes[30] as i8,
            airTemperature                  : bytes[31] as i8,
            totalLaps                       : bytes[32],
            trackLength                     : u16::from_le_bytes([bytes[33], bytes[34]]),
            sessionType                     : Session::from_u8(&bytes[35]),
            trackId                         : bytes[36] as i8,
            formula                         : Formula::from_u8(&bytes[37]),
            sessionTimeLeft                 : u16::from_le_bytes([bytes[38], bytes[39]]),
            sessionDuration                 : u16::from_le_bytes([bytes[40], bytes[41]]),
            pitSpeedLimit                   : bytes[42],
            gamePaused                      : bytes[43],
            isSpectating                    : bytes[44],
            spectatorCarIndex               : bytes[45],
            sliProNativeSupport             : SLIPro::from_u8(&bytes[46]),
            numMarshalZones                 : bytes[47],
            marshalZones                    : Self::marshalZone(&bytes[48..153]),
            safetyCarStatus                 : SafetyCar::from_u8(&bytes[153]),
            networkGame                     : NetworkGame::from_u8(&bytes[154]),
            numWeatherForecasts             : bytes[155],
            weatherForecast                 : Self::weatherForecast(&bytes[156..668]),
            forecastAccuracy                : Accuracy::from_u8(&bytes[668]),
            aiDifficulty                    : bytes[669],
            seasonLinkIdentifier            : u32::from_le_bytes([bytes[670], bytes[671], bytes[672], bytes[673]]),
            weekendLinkIdentifier           : u32::from_le_bytes([bytes[674], bytes[675], bytes[676], bytes[677]]),
            sessionLinkIdentifier           : u32::from_le_bytes([bytes[678], bytes[679], bytes[680], bytes[681]]),
            pitStopWindowIdealLap           : bytes[682],
            pitStopWindowLatestLap          : bytes[683],
            pitStopRejoinPosition           : bytes[684],
            steeringAssist                  : Assist::from_u8(&bytes[685]),
            brakingAssist                   : bytes[686],
            gearboxAssist                   : bytes[687],
            pitAssist                       : Assist::from_u8(&bytes[688]),
            pitReleaseAssist                : Assist::from_u8(&bytes[689]),
            ERSAssist                       : Assist::from_u8(&bytes[690]),
            DRSAssist                       : Assist::from_u8(&bytes[691]),
            dynamicRacingLine               : bytes[692],
            dynamicRacingLineType           : bytes[693],
            gameMode                        : bytes[694],
            ruleSet                         : bytes[695],
            timeOfDay                       : u32::from_le_bytes([bytes[696], bytes[697], bytes[698], bytes[699]]),
            sessionLength                   : SessionLength::from_u8(&bytes[700]),
            speedUnitsLeadPlayer            : bytes[701],
            temperatureUnitsLeadPlayer      : bytes[702],
            speedUnitsSecondaryPlayer       : bytes[703],
            temperatureUnitsSecondaryPlayer : bytes[704],
            numSafetyCarPeriods             : bytes[705],
            numVirtualSafetyCarPeriods      : bytes[706],
            numRedFlagPeriods               : bytes[707],
            equalCarPerformance             : bytes[708],
            recoveryMode                    : bytes[709],
            flashbackLimit                  : bytes[710],
            surfaceType                     : bytes[711],
            lowFuelMode                     : bytes[712],
            raceStarts                      : bytes[713],
            tyreTemperature                 : bytes[714],
            pitLaneTyreSim                  : bytes[715],
            carDamage                       : bytes[716],
            carDamageRate                   : bytes[717],
            collisions                      : bytes[718],
            collisionsOffForFirstLapOnly    : bytes[719],
            mpUnsafePitRelease              : bytes[720],
            mpOffForGriefing                : bytes[721],
            cornerCuttingStringency         : bytes[722],
            parcFermeRules                  : bytes[723],
            pitStopExperience               : bytes[724],
            safetyCar                       : bytes[725],
            safetyCarExperience             : bytes[726],
            formationLap                    : bytes[727],
            formationLapExperience          : bytes[728],
            redFlags                        : bytes[729],
            affectsLicenceLevelSolo         : bytes[730],
            affectsLicenceLevelMP           : bytes[731],
            numSessionsInWeekend            : bytes[732],
            weekendStructure                : [bytes[733], bytes[734], bytes[735], bytes[736], bytes[737], bytes[738], bytes[739], bytes[740], bytes[741], bytes[742], bytes[743], bytes[744]],
            sector2LapDistanceStart         : f32::from_le_bytes([bytes[745], bytes[746], bytes[747], bytes[748]]),
            sector3LapDistanceStart         : f32::from_le_bytes([bytes[749], bytes[750], bytes[751], bytes[752]]),
        }
    }

    pub fn marshalZone(bytes: &[u8]) -> [MarshalZone; 21]
    {
        let mut mz = [MarshalZone::default(); 21];

        let size = size_of::<MarshalZone>();

        for (i, mz) in mz.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *mz = MarshalZone::unpack(&bytes[s..e]);
        }

        mz
    }

    pub fn weatherForecast(bytes: &[u8]) -> [WeatherForecast; 64]
    {
        let mut wf = [WeatherForecast::default(); 64];

        let size = size_of::<WeatherForecast>();

        for (i, wf) in wf.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *wf = WeatherForecast::unpack(&bytes[s..e]);
        }

        wf
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Formula {
    Modern = 0,
    Classic = 1,
    Formula2 = 2,
    Generic = 3,
    Beta = 4,
    Supercars = 5,
    Esports = 6,
    Formula22021 = 7,
    #[default]
    Poisoned = 255
}

impl Formula
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => Formula::Modern,
            1 => Formula::Classic,
            2 => Formula::Formula2,
            3 => Formula::Generic,
            4 => Formula::Beta,
            5 => Formula::Supercars,
            6 => Formula::Esports,
            7 => Formula::Formula22021,
            _ => Formula::Poisoned,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum SLIPro {
    Inactive = 0,
    Active = 1,
    #[default]
    Poisoned = 255
}

impl SLIPro
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => SLIPro::Inactive,
            1 => SLIPro::Active,
            _ => SLIPro::Poisoned,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum NetworkGame {
    Offline = 0,
    Online = 1,
    #[default]
    Poisoned = 255
}

impl NetworkGame
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => NetworkGame::Offline,
            1 => NetworkGame::Online,
            _ => NetworkGame::Poisoned,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Accuracy {
    Perfect = 0,
    Approximate = 1,
    #[default]
    Poisoned = 255
}

impl Accuracy {
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => Accuracy::Perfect,
            1 => Accuracy::Approximate,
            _ => Accuracy::Poisoned,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Assist {
    Off = 0,
    On = 1,
    #[default]
    Poisoned = 255
}

impl Assist
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => Assist::Off,
            1 => Assist::On,
            _ => Assist::Poisoned
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum SafetyCar {
    Ready = 0,
    Deployed = 1,
    Virtual = 2,
    FormationLap = 3,
    #[default]
    Poisoned = 255
}

impl SafetyCar
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => SafetyCar::Ready,
            1 => SafetyCar::Deployed,
            2 => SafetyCar::Virtual,
            3 => SafetyCar::FormationLap,
            _ => SafetyCar::Poisoned,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum SessionLength {
    None = 0,
    VeryShort = 2,
    Short = 3,
    Medium = 4,
    MediumLong = 5,
    Long = 6,
    Full = 7,
    #[default]
    Poisoned = 255
}

impl SessionLength
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => SessionLength::None,
            2 => SessionLength::VeryShort,
            3 => SessionLength::Short,
            4 => SessionLength::Medium,
            5 => SessionLength::MediumLong,
            6 => SessionLength::Long,
            7 => SessionLength::Full,
            _ => SessionLength::Poisoned,
        }
    }
}


/// # Lap Data Packet
/// The lap data packet gives details of all the cars in the session.
/// Frequency: Rate as specified in menus
/// Size: 1285 bytes
/// Version: 1
#[repr(C, packed)] // Size: 56 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct Car
{
    pub lastLapTimeInMS: u32,             // Last lap time in milliseconds
    pub currentLapTimeInMS: u32,          // Current time around the lap in milliseconds
    pub sector1TimeMSPart: u16,           // Sector 1 time milliseconds part
    pub sector1TimeMinutesPart: u8,       // Sector 1 whole minute part
    pub sector2TimeMSPart: u16,           // Sector 2 time milliseconds part
    pub sector2TimeMinutesPart: u8,       // Sector 2 whole minute part
    pub deltaToCarInFrontMSPart: u16,     // Time delta to car in front milliseconds part
    pub deltaToCarInFrontMinutesPart: u8, // Time delta to car in front whole minute part
    pub deltaToRaceLeaderMSPart: u16,     // Time delta to race leader milliseconds part
    pub deltaToRaceLeaderMinutesPart: u8, // Time delta to race leader whole minute part
    pub lapDistance: f32,                 // Distance vehicle is around current lap in metres – could be negative if line hasn’t been crossed yet
    pub totalDistance: f32,               // Total distance travelled in session in metres – could be negative if line hasn’t been crossed yet
    pub safetyCarDelta: f32,              // Delta in seconds for safety car
    pub racePosition: u8,                 // Car race position
    pub currentLapNum: u8,                // Current lap number
    pub pitStatus: PitStatus,             // 0 = none, 1 = pitting, 2 = in pit area
    pub numPitStops: u8,                  // Number of pit stops taken in this race
    pub sector: u8,                       // 0 = sector1, 1 = sector2, 2 = sector3
    pub currentLapInvalid: u8,            // Current lap invalid - 0 = valid, 1 = invalid
    pub penalties: u8,                    // Accumulated time penalties in seconds to be added
    pub totalWarnings: u8,                // Accumulated number of warnings issued
    pub cornerCuttingWarnings: u8,        // Accumulated number of corner cutting warnings issued
    pub numUnservedDriveThroughPens: u8,  // Num drive through pens left to serve
    pub numUnservedStopGoPens: u8,        // Num stop go pens left to serve
    pub gridPosition: u8,                 // Grid position the vehicle started the race in
    pub driverStatus: CarState,           // Status of driver - 0 = in garage, 1 = flying lap, 2 = in lap, 3 = out lap, 4 = on track
    pub resultStatus: ResultStatus,       // Result status - 0 = invalid, 1 = inactive, 2 = active, 3 = finished, 4 = didnotfinish, 5 = disqualified, 6 = not classified, 7 = retired
    pub pitLaneTimerActive: u8,           // Pit lane timing, 0 = inactive, 1 = active
    pub pitLaneTimeInLaneInMS: u16,       // If active, the current time spent in the pit lane in ms
    pub pitStopTimerInMS: u16,            // Time of the actual pit stop in ms
    pub pitStopShouldServePen: u8,        // Whether the car should serve a penalty at this stop
    pub speedTrapFastestSpeed: f32,       // Fastest speed through speed trap for this car in kmph
    pub speedTrapFastestLap: u8,          // Lap no the fastest speed was achieved, 255 = not set
}

impl Car
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            lastLapTimeInMS: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            currentLapTimeInMS: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            sector1TimeMSPart: u16::from_le_bytes([bytes[8], bytes[9]]),
            sector1TimeMinutesPart: bytes[10],
            sector2TimeMSPart: u16::from_le_bytes([bytes[11], bytes[12]]),
            sector2TimeMinutesPart: bytes[13],
            deltaToCarInFrontMSPart: u16::from_le_bytes([bytes[14], bytes[15]]),
            deltaToCarInFrontMinutesPart: bytes[16],
            deltaToRaceLeaderMSPart: u16::from_le_bytes([bytes[17], bytes[18]]),
            deltaToRaceLeaderMinutesPart: bytes[19],
            lapDistance: f32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            totalDistance: f32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            safetyCarDelta: f32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
            racePosition: bytes[32],
            currentLapNum: bytes[33],
            pitStatus: PitStatus::from_u8(bytes[34]),
            numPitStops: bytes[35],
            sector: bytes[36],
            currentLapInvalid: bytes[37],
            penalties: bytes[38],
            totalWarnings: bytes[39],
            cornerCuttingWarnings: bytes[40],
            numUnservedDriveThroughPens: bytes[41],
            numUnservedStopGoPens: bytes[42],
            gridPosition: bytes[43],
            driverStatus: CarState::from_u8(bytes[44]),
            resultStatus: ResultStatus::from_u8(bytes[45]),
            pitLaneTimerActive: bytes[46],
            pitLaneTimeInLaneInMS: u16::from_le_bytes([bytes[47], bytes[48]]),
            pitStopTimerInMS: u16::from_le_bytes([bytes[49], bytes[50]]),
            pitStopShouldServePen: bytes[51],
            speedTrapFastestSpeed: f32::from_le_bytes([bytes[52], bytes[53], bytes[54], bytes[55]]),
            speedTrapFastestLap: bytes[56],
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum PitStatus {
    None = 0,
    Pitting = 1,
    InPitArea = 2,
    #[default]
    Poisoned = 255,
}

impl PitStatus
{
    pub fn from_u8(byte: u8) -> Self
    {
        match byte
        {
            0 => PitStatus::None,
            1 => PitStatus::Pitting,
            2 => PitStatus::InPitArea,
            _ => PitStatus::Poisoned,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum CarState {
    InGarage = 0,
    OnFlyingLap = 1,
    InLap = 2,
    OutLap = 3,
    OnTrack = 4,
    #[default]
    Poisoned = 255,
}

impl CarState {
    pub fn from_u8(byte: u8) -> Self
    {
        match byte
        {
            0 => CarState::InGarage,
            1 => CarState::OnFlyingLap,
            2 => CarState::InLap,
            3 => CarState::OutLap,
            4 => CarState::OnTrack,
            _ => CarState::Poisoned,
        }
    }
}

impl fmt::Display for CarState
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self
        {
            CarState::InGarage    => write!(f, "Pits"),
            CarState::OnFlyingLap => write!(f, "Fast"),
            CarState::InLap       => write!(f, "In"),
            CarState::OutLap      => write!(f, "Out"),
            CarState::OnTrack     => write!(f, "Race"),
            _                     => write!(f, "?")
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum ResultStatus {
    #[default]
    Invalid = 0,
    Inactive = 1,
    Active = 2,
    Finished = 3,
    DidNotFinish = 4,
    Disqualified = 5,
    NotClassified = 6,
    Retired = 7,
}

impl ResultStatus {
    pub fn from_u8(byte: u8) -> Self
    {
        match byte
        {
            0 => ResultStatus::Invalid,
            1 => ResultStatus::Inactive,
            2 => ResultStatus::Active,
            3 => ResultStatus::Finished,
            4 => ResultStatus::DidNotFinish,
            5 => ResultStatus::Disqualified,
            6 => ResultStatus::NotClassified,
            7 => ResultStatus::Retired,
            _ => ResultStatus::Invalid,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct PacketLap
{
    pub header: Header,                 // 24 Bytes - Header

    pub cars: [Car; 22],                // Lap data for all cars on track

    pub timeTrialPBCarIdx: u8,          // Index of Personal Best car in time trial (255 if invalid)
    pub timeTrialRivalCarIdx: u8,       // Index of Rival car in time trial (255 if invalid)
}

impl PacketLap
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        let sz_hdr = size_of::<Header>();
        let sz_lap = size_of::<Car>();

        Self {
            header: Header::unpack(bytes),

            cars: Self::lap(&bytes[sz_hdr .. sz_hdr + (sz_lap * 22)]),

            timeTrialPBCarIdx: bytes[sz_hdr + (sz_lap * 22) + 1],
            timeTrialRivalCarIdx: bytes[sz_hdr + (sz_lap * 22) + 2],
        }
    }

    pub fn lap(bytes: &[u8]) -> [Car; 22]
    {
        let mut cars = [Car::default(); 22];

        let size = size_of::<Car>();

        for (idx, car) in cars.iter_mut().enumerate()
        {
            let s = idx * size;
            let e = s + size;

            *car = Car::unpack(&bytes[s..e]);
        }

        cars
    }
}

/// # Event Packet
/// This packet gives details of events that happen during the course of a session.
/// Frequency: When the event occurs
/// Size: 40 bytes
/// Version: 1
/// The event details packet is different for each type of event.
/// Make sure only the correct type is interpreted.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union EventDetails
{
    pub sessionStarted: SessionStarted,
    pub sessionEnded: SessionEnded,
    pub fastestLap: FastestLap,
    pub retirement: Retirement,
    pub drsEnabled: DRSEnabled,
    pub drsDisabled: DRSDisabled,
    pub teamMateInPits: TeamMateInPits,
    pub chequeredFlag: ChequeredFlag,
    pub raceWinner: RaceWinner,
    pub penalty: Penalty,
    pub speedTrap: SpeedTrap,
    pub startLights: StartLights,
    pub lightsOut: LightsOut,
    pub driveThroughPenaltyServed: DriveThroughPenaltyServed,
    pub stopGoPenaltyServed: StopGoPenaltyServed,
    pub flashback: Flashback,
    pub buttons: Buttons,
    pub unknownTag: [u8; 4]
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SessionStarted {}
impl SessionStarted
{
    pub fn unpack() -> Self
    {
        Self {}
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SessionEnded {}
impl SessionEnded
{
    pub fn unpack() -> Self
    {
        Self {}
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct FastestLap
{
    pub vehicleIdx: u8,                 // Vehicle index of car achieving fastest lap
    pub lapTime: f32,                   // Lap time is in seconds
}

impl FastestLap
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            vehicleIdx: bytes[0],
            lapTime: f32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Retirement
{
    pub vehicleIdx: u8,                 // Vehicle index of car retiring
}

impl Retirement
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            vehicleIdx: bytes[0],
        }
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct DRSEnabled {}
impl DRSEnabled
{
    pub fn unpack() -> Self
    {
        Self {}
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct DRSDisabled {}
impl DRSDisabled
{
    pub fn unpack() -> Self
    {
        Self {}
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TeamMateInPits
{
    pub vehicleIdx: u8,                 // Vehicle index of team mate
}

impl TeamMateInPits
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            vehicleIdx: bytes[0],
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct ChequeredFlag {}
impl ChequeredFlag
{
    pub fn unpack() -> Self
    {
        Self {}
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct RaceWinner
{
    pub vehicleIdx: u8,                 // Vehicle index of the race winner
}

impl RaceWinner
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            vehicleIdx: bytes[0],
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Penalty
{
    pub penaltyType: u8,                // Penalty type – see Appendices
    pub infringementType: u8,           // Infringement type – see Appendices
    pub vehicleIdx: u8,                 // Vehicle index of the car the penalty is applied to
    pub otherVehicleIdx: u8,            // Vehicle index of the other car involved
    pub time: u8,                       // Time gained, or time spent doing action in seconds
    pub lapNum: u8,                     // Lap the penalty occurred on
    pub placesGained: u8,               // Number of places gained by this
}

impl Penalty
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            penaltyType: bytes[0],
            infringementType: bytes[1],
            vehicleIdx: bytes[2],
            otherVehicleIdx: bytes[3],
            time: bytes[4],
            lapNum: bytes[5],
            placesGained: bytes[6],
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SpeedTrap
{
    pub vehicleIdx: u8,                 // Vehicle index of the vehicle triggering speed trap
    pub speed: f32,                     // Top speed achieved in kilometres per hour
    pub isOverallFastestInSession: u8,  // Overall fastest speed in session = 1, otherwise 0
    pub isDriverFastestInSession: u8,   // Fastest speed for driver in session = 1, otherwise 0
    pub fastestVehicleIdxInSession: u8, // Vehicle index of the vehicle that is the fastest in this session
    pub fastestSpeedInSession: f32,     // Speed of the vehicle that is the fastest in this session
}

impl SpeedTrap
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            vehicleIdx                : bytes[ 0],
            speed                     : f32::from_le_bytes([bytes[ 1], bytes[ 2], bytes[ 3], bytes[ 4]]),
            isOverallFastestInSession : bytes[ 5],
            isDriverFastestInSession  : bytes[ 6],
            fastestVehicleIdxInSession: bytes[ 7],
            fastestSpeedInSession     : f32::from_le_bytes([bytes[ 8], bytes[ 9], bytes[10], bytes[11]]),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct StartLights
{
    pub numLights: u8,                  // Number of lights showing
}

impl StartLights
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            numLights: bytes[0],
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct LightsOut {}
impl LightsOut
{
    pub fn unpack() -> Self
    {
        Self {}
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct DriveThroughPenaltyServed
{
    pub vehicleIdx: u8,                 // Vehicle index of the vehicle serving drive through
}

impl DriveThroughPenaltyServed
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            vehicleIdx: bytes[0],
        }
    }

}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct StopGoPenaltyServed
{
    pub vehicleIdx: u8,                 // Vehicle index of the vehicle serving stop go
}

impl StopGoPenaltyServed
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            vehicleIdx: bytes[0],
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Flashback
{
    pub flashbackFrameIdentifier: u32,  // Frame identifier flashed back to
    pub flashbackSessionTime: f32,      // Session time flashed back to
}

impl Flashback
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            flashbackFrameIdentifier: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            flashbackSessionTime    : f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Buttons
{
    pub buttonStatus: u32,              // Bit flags specifying which buttons are being pressed currently - see appendices
}

impl Buttons
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            buttonStatus: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
pub struct EventTag
{
    pub tag: [u8; 4]
}

impl EventTag
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            tag: [bytes[0], bytes[1], bytes[2], bytes[3]]
        }
    }
}

#[repr(C, packed)] // Size: 24 + 8 + (Depends) Bytes
#[derive(Clone, Copy)]
pub struct PacketEvent
{
    pub header: Header,                 // 24 Bytes - Header

    pub eventStringCode: EventTag,      // u8 * 4 - Event string code, see below
    pub eventType: EventType,           // Tag for the Event Type.
    pub eventDetails: EventDetails,     // Depends - Event details - should be interpreted differently for each type
}

impl fmt::Debug for PacketEvent
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PacketEvent")
         .field(         "header", &self.header)
         .field("eventStringCode", &std::str::from_utf8(&self.eventStringCode.tag).unwrap().trim_end_matches('\0'))
         .field(   "eventDetails", &"God Only Knows")
         .finish()
    }
}

impl PacketEvent
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        let eventTag: EventTag = EventTag::unpack(&bytes[29..34]);

        Self {
            header: Header::unpack(bytes),

            eventStringCode: eventTag,
            eventType: match &eventTag.tag {
                b"SSTA" => EventType::SessionStarted,
                b"SEND" => EventType::SessionEnded,
                b"FTLP" => EventType::FastestLap,
                b"RTMT" => EventType::Retirement,
                b"DRSE" => EventType::DRSEnabled,
                b"DRSD" => EventType::DRSDisabled,
                b"TMPT" => EventType::TeamMateInPits,
                b"CHQF" => EventType::ChequeredFlag,
                b"RCWN" => EventType::RaceWinner,
                b"PENA" => EventType::Penalty,
                b"SPTP" => EventType::SpeedTrap,
                b"STLG" => EventType::StartLights,
                b"LGOT" => EventType::LightsOut,
                b"DTSV" => EventType::DriveThroughPenaltyServed,
                b"SGSV" => EventType::StopGoPenaltyServed,
                b"FLBK" => EventType::Flashback,
                b"BUTN" => EventType::Buttons,
                _ => EventType::Poisoned,
            },
            eventDetails: match &eventTag.tag {
                b"SSTA" => EventDetails {
                        // Sent when the session starts
                        sessionStarted: SessionStarted::unpack(),
                    },
                b"SEND" => EventDetails {
                        // Sent when the session ends
                        sessionEnded: SessionEnded::unpack(),
                    },
                b"FTLP" => EventDetails {
                        // When a driver achieves the fastest lap
                        fastestLap: FastestLap::unpack(&bytes[28..])
                    },
                b"RTMT" => EventDetails {
                        // When a driver retires
                        retirement: Retirement::unpack(&bytes[28..])
                    },
                b"DRSE" => EventDetails {
                        // Race control have enabled DRS
                        drsEnabled: DRSEnabled::unpack(),
                    },
                b"DRSD" => EventDetails {
                        // Race control have disabled DRS
                        drsDisabled: DRSDisabled::unpack(),
                    },
                b"TMPT" => EventDetails {
                        // Your team mate has entered the pits
                        teamMateInPits: TeamMateInPits::unpack(&bytes[28..])
                    },
                b"CHQF" => EventDetails {
                        // The chequered flag has been waved
                        chequeredFlag: ChequeredFlag::unpack(),
                    },
                b"RCWN" => EventDetails {
                        // The race winner is announced
                        raceWinner: RaceWinner::unpack(&bytes[28..])
                    },
                b"PENA" => EventDetails {
                        // A penalty has been issued – details in event
                        penalty: Penalty::unpack(&bytes[28..])
                    },
                b"SPTP" => EventDetails {
                        // Speed trap has been triggered by fastest speed
                        speedTrap: SpeedTrap::unpack(&bytes[28..])
                    },
                b"STLG" => EventDetails {
                        // Start lights – number shown
                        startLights: StartLights::unpack(&bytes[28..])
                    },
                b"LGOT" => EventDetails {
                        // Lights out
                        lightsOut: LightsOut::unpack(),
                    },
                b"DTSV" => EventDetails {
                        // Drive through penalty served
                        driveThroughPenaltyServed: DriveThroughPenaltyServed::unpack(&bytes[28..])
                    },
                b"SGSV" => EventDetails {
                        // Stop go penalty served
                        stopGoPenaltyServed: StopGoPenaltyServed::unpack(&bytes[28..])
                    },
                b"FLBK" => EventDetails {
                        // Flashback activated
                        flashback: Flashback::unpack(&bytes[28..])
                    },
                b"BUTN" => EventDetails {
                        // Button status changed
                        buttons: Buttons::unpack(&bytes[28..])
                    },
                _ => {
                    println!("Unhandled Event: {:?}", eventTag);
                    EventDetails {
                        unknownTag: eventTag.tag
                    }
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(u8)]
pub enum EventType {
    SessionStarted,
    SessionEnded,
    FastestLap,
    Retirement,
    DRSEnabled,
    DRSDisabled,
    TeamMateInPits,
    ChequeredFlag,
    RaceWinner,
    Penalty,
    SpeedTrap,
    StartLights,
    LightsOut,
    DriveThroughPenaltyServed,
    StopGoPenaltyServed,
    Flashback,
    Buttons,
    #[default]
    Poisoned,
}

/// # Participants Packet
/// This is a list of participants in the race. If the vehicle is controlled by AI, then the name will be the driver name. If this is a multiplayer game, the names will be the Steam Id on PC, or the LAN name if appropriate.
/// N.B. on Xbox One, the names will always be the driver name, on PS4 the name will be the LAN name if playing a LAN game, otherwise it will be the driver name.
/// The array should be indexed by vehicle index.
/// Frequency: Every 5 seconds
/// Size: 1284 bytes
/// Version: 1
#[repr(C, packed)] // Size: 56 Bytes
#[derive(Clone, Copy, Default)]
pub struct Participant
{
    pub aiControlled: u8,                   // Whether the vehicle is AI (1) or Human (0) controlled
    pub driverId: u8,                       // Driver id - see appendix, 255 if network human
    pub networkId: u8,                      // Network id – unique identifier for network players
    pub teamId: u8,                         // Team id - see appendix
    pub myTeam: u8,                         // My team flag – 1 = My Team, 0 = otherwise
    pub raceNumber: u8,                     // Race number of the car
    pub nationality: u8,                    // Nationality of the driver
    pub name: [u8; 32],                     // Name of participant in UTF-8 format – null terminated
                                            // Will be truncated with … (U+2026) if too long
    pub yourTelemetry: u8,                  // The player's UDP setting, 0 = restricted, 1 = public
    pub showOnlineNames: u8,                // The player's show online names setting, 0 = off, 1 = on
    pub techLevel: u16,                     // F1 World tech level
    pub platform: u8,                       // 1 = Steam, 3 = PlayStation, 4 = Xbox, 6 = Origin, 255 = unknown
    pub numColours: u8,                     // Number of colours valid for this car
    pub liveryColours: [LiveryColour; 4],   // Colours for the car
}

impl Participant
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            aiControlled: bytes[0],
            driverId: bytes[1],
            networkId: bytes[2],
            teamId: bytes[3],
            myTeam: bytes[4],
            raceNumber: bytes[5],
            nationality: bytes[6],
            name: match bytes[7..39].try_into()
                    {
                        Ok(str) => str,
                        Err(err) => {
                            dbg!(err);
                            [0; 32]
                        }
                    },
            yourTelemetry: bytes[39],
            showOnlineNames: bytes[40],
            techLevel: u16::from_le_bytes([bytes[41], bytes[42]]),
            platform: bytes[43],
            numColours: bytes[44],
            liveryColours: LiveryColour::unpack(&bytes[45..])
        }
    }

    pub fn name_to_string(&self) -> String
    {
        std::str::from_utf8(&self.name).unwrap().trim_end_matches('\0').to_string()
    }
}

/// RGB value of a colour
#[derive(Clone, Copy, Default)]
pub struct LiveryColour
{
    pub r: u8,  // Red
    pub g: u8,  // Green
    pub b: u8,  // Blue
}

impl LiveryColour {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
        }
    }

    pub fn unpack(bytes: &[u8]) -> [Self; 4] {
        [
            Self::from_bytes(&bytes[ 0.. 3]),
            Self::from_bytes(&bytes[ 3.. 6]),
            Self::from_bytes(&bytes[ 6.. 9]),
            Self::from_bytes(&bytes[ 9..12]),
        ]
    }
}

impl fmt::Debug for Participant
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Participant")
         .field( "aiControlled", &self.aiControlled)
         .field(     "driverId", &self.driverId)
         .field(    "networkId", &self.networkId)
         .field(       "teamId", &self.teamId)
         .field(       "myTeam", &self.myTeam)
         .field(   "raceNumber", &self.raceNumber)
         .field(  "nationality", &self.nationality)
         .field(         "name", &std::str::from_utf8(&self.name).unwrap().trim_end_matches('\0'))
         .field("yourTelemetry", &self.yourTelemetry)
         .finish()
    }
}

#[repr(C, packed)] // Size: 1257 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct PacketParticipants
{
    pub header: Header,                 // 24 Bytes - Header

    pub numActiveCars: u8,              // Number of active cars in the data – should match number of cars on HUD
    pub participants: [Participant; 22],
}

impl PacketParticipants
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        let sz_hdr = size_of::<Header>();
        Self {
            header: Header::unpack(bytes),

            numActiveCars: bytes[sz_hdr],
            participants: Self::participants(&bytes[sz_hdr + 1..]),
        }
    }

    pub fn participants(bytes: &[u8]) -> [Participant; 22]
    {
        let mut p = [Participant::default(); 22];
        let size = size_of::<Participant>();

        for (i, p) in p.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *p = Participant::unpack(&bytes[s..e]);
        }

        p
    }
}

/// # Car Setups Packet
/// This packet details the car setups for each vehicle in the session. Note that in multiplayer games, other player cars will appear as blank, you will only be able to see your car setup and AI cars.
/// Frequency: 2 per second
/// Size: 1102 bytes
/// Version: 1
#[repr(C, packed)] // Size: 49 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct CarSetup
{
    pub frontWing: u8,                  // Front wing aero
    pub rearWing: u8,                   // Rear wing aero
    pub onThrottle: u8,                 // Differential adjustment on throttle (percentage)
    pub offThrottle: u8,                // Differential adjustment off throttle (percentage)
    pub frontCamber: f32,               // Front camber angle (suspension geometry)
    pub rearCamber: f32,                // Rear camber angle (suspension geometry)
    pub frontToe: f32,                  // Front toe angle (suspension geometry)
    pub rearToe: f32,                   // Rear toe angle (suspension geometry)
    pub frontSuspension: u8,            // Front suspension
    pub rearSuspension: u8,             // Rear suspension
    pub frontAntiRollBar: u8,           // Front anti-roll bar
    pub rearAntiRollBar: u8,            // Front anti-roll bar
    pub frontSuspensionHeight: u8,      // Front ride height
    pub rearSuspensionHeight: u8,       // Rear ride height
    pub brakePressure: u8,              // Brake pressure (percentage)
    pub brakeBias: u8,                  // Brake bias (percentage)
    pub tyrePressure: Wheels,           // 16 Bytes - Tyre pressures in PSI
    pub ballast: u8,                    // Ballast
    pub fuelLoad: f32,                  // Fuel load
}

impl CarSetup
{
    pub fn unpack(bytes: &[u8]) -> Self {
        Self {
            frontWing            : bytes[ 0],
            rearWing             : bytes[ 1],
            onThrottle           : bytes[ 2],
            offThrottle          : bytes[ 3],
            frontCamber          : f32::from_le_bytes([bytes[ 4], bytes[ 5], bytes[ 6], bytes[ 7]]),
            rearCamber           : f32::from_le_bytes([bytes[ 8], bytes[ 9], bytes[10], bytes[11]]),
            frontToe             : f32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            rearToe              : f32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            frontSuspension      : bytes[20],
            rearSuspension       : bytes[21],
            frontAntiRollBar     : bytes[22],
            rearAntiRollBar      : bytes[23],
            frontSuspensionHeight: bytes[24],
            rearSuspensionHeight : bytes[25],
            brakePressure        : bytes[26],
            brakeBias            : bytes[27],
            tyrePressure         : Wheels::unpack(&bytes[28..44]),
            ballast              : bytes[44],
            fuelLoad             : f32::from_le_bytes([bytes[45], bytes[46], bytes[47], bytes[48]]),
        }
    }
}

#[repr(C, packed)] // Size: 1102 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct PacketCarSetups
{
    pub header: Header,                 // 24 Bytes - Header

    pub carSetups: [CarSetup; 22],
}

impl PacketCarSetups
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self
        {
            header: Header::unpack(bytes),

            carSetups: Self::carSetups(&bytes[size_of::<Header>()..])
        }
    }

    pub fn carSetups(bytes: &[u8]) -> [CarSetup; 22]
    {
        let mut cs = [CarSetup::default(); 22];
        let size = size_of::<CarSetup>();

        for (i, cs) in cs.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *cs = CarSetup::unpack(&bytes[s..e]);
        }

        cs
    }
}

/// # Car Telemetry Packet
/// This packet details telemetry for all the cars in the race. It details various values that would be recorded on the car such as speed, throttle application, DRS etc. Note that the rev light configurations are presented separately as well and will mimic real life driver preferences.
/// Frequency: Rate as specified in menus
/// Size: 1347 bytes
/// Version: 1
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Kph {
    pub kph: u16
}

impl Kph {
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self
        {
            kph: ((bytes[1] as u16) << 8) + bytes[0] as u16
        }
    }
    pub fn toMPH(&self) -> f32
    {
        self.kph as f32 * 0.621_371_2
    }
    pub fn toMPHString(&self) -> String
    {
        format!("{:.3}", self.toMPH())
    }
}

impl fmt::Display for Kph
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kph)
    }
}

#[repr(i8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Gear
{
    Reverse =-1,
    Neutral = 0,
    First   = 1,
    Second  = 2,
    Third   = 3,
    Forth   = 4,
    Fifth   = 5,
    Sixth   = 6,
    Seventh = 7,
    Eighth  = 8,
    #[default]
    Poisoned= 127,
}

impl Gear
{
    pub fn from_u8_to_i8(byte: &u8) -> Self
    {
        match *byte as i8
        {
            -1 => Gear::Reverse,
             0 => Gear::Neutral,
             1 => Gear::First,
             2 => Gear::Second,
             3 => Gear::Third,
             4 => Gear::Forth,
             5 => Gear::Fifth,
             6 => Gear::Sixth,
             7 => Gear::Seventh,
             8 => Gear::Eighth,
             _ => Gear::Poisoned,
        }
    }
}

impl fmt::Display for Gear
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self
        {
            Gear::Reverse  => write!(f, "R"),
            Gear::Neutral  => write!(f, "N"),
            Gear::First    => write!(f, "1"),
            Gear::Second   => write!(f, "2"),
            Gear::Third    => write!(f, "3"),
            Gear::Forth    => write!(f, "4"),
            Gear::Fifth    => write!(f, "5"),
            Gear::Sixth    => write!(f, "6"),
            Gear::Seventh  => write!(f, "7"),
            Gear::Eighth   => write!(f, "8"),
            Gear::Poisoned => write!(f, "?"),
        }
    }
}

#[repr(u16)]
#[derive(Debug, Default, Clone, Copy)]
pub enum LEDs
{
    #[default]
    None         = 0b0000000000000000,
    One          = 0b0000000000000001,
    Two          = 0b0000000000000010,
    Three        = 0b0000000000000100,
    Four         = 0b0000000000001000,
    Five         = 0b0000000000010000,
    Six          = 0b0000000000100000,
    Seven        = 0b0000000001000000,
    Eight        = 0b0000000010000000,
    Nine         = 0b0000000100000000,
    Ten          = 0b0000001000000000,
    Eleven       = 0b0000010000000000,
    Twelve       = 0b0000100000000000,
    Thriteen     = 0b0001000000000000,
    Fourteen     = 0b0010000000000000,
    Fifthteen    = 0b0100000000000000,
}

#[repr(C)] // Size: 2 Bytes
#[derive(Default, Clone, Copy)]
pub struct RevLights
{
    pub LEDs: u16,
}

impl RevLights
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            LEDs: ((bytes[1] as u16) << 8) + bytes[0] as u16
        }
    }
}

impl fmt::Display for RevLights
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            // Green
            if (self.LEDs & LEDs::One          as u16) != 0 { "O".green()  } else { "o".white() },
            if (self.LEDs & LEDs::Two          as u16) != 0 { "O".green()  } else { "o".white() },
            if (self.LEDs & LEDs::Three        as u16) != 0 { "O".green()  } else { "o".white() },
            if (self.LEDs & LEDs::Four         as u16) != 0 { "O".green()  } else { "o".white() },
            if (self.LEDs & LEDs::Five         as u16) != 0 { "O".green()  } else { "o".white() },
            // Red
            if (self.LEDs & LEDs::Six          as u16) != 0 { "O".red()    } else { "o".white() },
            if (self.LEDs & LEDs::Seven        as u16) != 0 { "O".red()    } else { "o".white() },
            if (self.LEDs & LEDs::Eight        as u16) != 0 { "O".red()    } else { "o".white() },
            if (self.LEDs & LEDs::Nine         as u16) != 0 { "O".red()    } else { "o".white() },
            if (self.LEDs & LEDs::Ten          as u16) != 0 { "O".red()    } else { "o".white() },
            // Purple
            if (self.LEDs & LEDs::Eleven       as u16) != 0 { "O".purple() } else { "o".white() },
            if (self.LEDs & LEDs::Twelve       as u16) != 0 { "O".purple() } else { "o".white() },
            if (self.LEDs & LEDs::Thriteen     as u16) != 0 { "O".purple() } else { "o".white() },
            if (self.LEDs & LEDs::Fourteen     as u16) != 0 { "O".purple() } else { "o".white() },
            if (self.LEDs & LEDs::Fifthteen    as u16) != 0 { "O".purple() } else { "o".white() },
        )
    }
}

impl fmt::Debug for RevLights
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{} ({:#016b})",
            // Green
            if (self.LEDs & LEDs::One          as u16) != 0 { "O".green()  } else { "o".white() },
            if (self.LEDs & LEDs::Two          as u16) != 0 { "O".green()  } else { "o".white() },
            if (self.LEDs & LEDs::Three        as u16) != 0 { "O".green()  } else { "o".white() },
            if (self.LEDs & LEDs::Four         as u16) != 0 { "O".green()  } else { "o".white() },
            if (self.LEDs & LEDs::Five         as u16) != 0 { "O".green()  } else { "o".white() },
            // Red
            if (self.LEDs & LEDs::Six          as u16) != 0 { "O".red()    } else { "o".white() },
            if (self.LEDs & LEDs::Seven        as u16) != 0 { "O".red()    } else { "o".white() },
            if (self.LEDs & LEDs::Eight        as u16) != 0 { "O".red()    } else { "o".white() },
            if (self.LEDs & LEDs::Nine         as u16) != 0 { "O".red()    } else { "o".white() },
            if (self.LEDs & LEDs::Ten          as u16) != 0 { "O".red()    } else { "o".white() },
            // Purple
            if (self.LEDs & LEDs::Eleven       as u16) != 0 { "O".purple() } else { "o".white() },
            if (self.LEDs & LEDs::Twelve       as u16) != 0 { "O".purple() } else { "o".white() },
            if (self.LEDs & LEDs::Thriteen     as u16) != 0 { "O".purple() } else { "o".white() },
            if (self.LEDs & LEDs::Fourteen     as u16) != 0 { "O".purple() } else { "o".white() },
            if (self.LEDs & LEDs::Fifthteen    as u16) != 0 { "O".purple() } else { "o".white() },
            // Value
            self.LEDs
        )
    }
}

#[repr(C, packed)] // Size: 60 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct CarTelemetry
{
    pub speed: Kph,                         // Speed of car in kilometres per hour
    pub throttle: f32,                      // Amount of throttle applied (0.0 to 1.0)
    pub steer: f32,                         // Steering (-1.0 (full lock left) to 1.0 (full lock right))
    pub brake: f32,                         // Amount of brake applied (0.0 to 1.0)
    pub clutch: u8,                         // Amount of clutch applied (0 to 100)
    pub gear: Gear,                         // i8 1 Byte - Gear selected (1-8, N=0, R=-1)
    pub engineRPM: u16,                     // Engine RPM
    pub drs: u8,                            // 0 = off, 1 = on
    pub revLightsPercent: u8,               // Rev lights indicator (percentage)
    pub revLightsBitValue: RevLights,       // u16 2 Bytes - Rev lights (bit 0 = leftmost LED, bit 14 = rightmost LED)
    pub brakesTemperature: [u16; 4],        // Brakes temperature (celsius)
    pub tyresSurfaceTemperature: [u8; 4],   // Tyres surface temperature (celsius)
    pub tyresInnerTemperature: [u8; 4],     // Tyres inner temperature (celsius)
    pub engineTemperature: u16,             // Engine temperature (celsius)
    pub tyresPressure: [f32; 4],            // Tyres pressure (PSI)
    pub surfaceType: [u8; 4],               // Driving surface, see appendices
}

impl CarTelemetry
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            speed                  : Kph::unpack(&bytes[ 0.. 2]),
            throttle               : f32::from_le_bytes([bytes[ 2], bytes[ 3], bytes[ 4], bytes[ 5]]),
            steer                  : f32::from_le_bytes([bytes[ 6], bytes[ 7], bytes[ 8], bytes[ 9]]),
            brake                  : f32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]),
            clutch                 : bytes[14],
            gear                   : Gear::from_u8_to_i8(&bytes[15]),
            engineRPM              : u16::from_le_bytes([bytes[16], bytes[17]]),
            drs                    : bytes[18],
            revLightsPercent       : bytes[19],
            revLightsBitValue      : RevLights::unpack(&bytes[20..22]),
            brakesTemperature      : [
                                     u16::from_le_bytes([bytes[22], bytes[23]]),
                                     u16::from_le_bytes([bytes[24], bytes[25]]),
                                     u16::from_le_bytes([bytes[26], bytes[27]]),
                                     u16::from_le_bytes([bytes[28], bytes[29]]),
            ],
            tyresSurfaceTemperature: [bytes[30], bytes[31], bytes[32], bytes[33]],
            tyresInnerTemperature  : [bytes[34], bytes[35], bytes[36], bytes[37]],
            engineTemperature      : u16::from_le_bytes([bytes[38], bytes[39]]),
            tyresPressure          : [
                                     f32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]),
                                     f32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]),
                                     f32::from_le_bytes([bytes[48], bytes[49], bytes[50], bytes[50]]),
                                     f32::from_le_bytes([bytes[52], bytes[53], bytes[54], bytes[55]]),
            ],
            surfaceType            : [bytes[56], bytes[57], bytes[58], bytes[59]],
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum MFDPanel
{
    Setup = 0,
    Pits = 1,
    Damage = 2,
    Engine = 3,
    Temperatures = 4,
    #[default]
    Poisoned = 254,
    Closed = 255,
}

impl MFDPanel
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
              0 => MFDPanel::Setup,
              1 => MFDPanel::Pits,
              2 => MFDPanel::Damage,
              3 => MFDPanel::Engine,
              4 => MFDPanel::Temperatures,
            255 => MFDPanel::Closed,
              _ => MFDPanel::Poisoned,
        }
    }
}

#[repr(C, packed)] // Size: 1352 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct PacketCarTelemetry
{
    pub header: Header,                     // 24 Bytes - Header

    pub carTelemetry: [CarTelemetry; 22],   // 60 * 22 = 1320 Bytes

    pub mfdFirstPlayer: MFDPanel,           // u8 - Index of MFD panel open - 255 = MFD closed Single player, race – 0 = Car setup, 1 = Pits 2 = Damage, 3 =  Engine, 4 = Temperatures May vary depending on game mode
    pub mfdSecondaryPlayer: MFDPanel,       // u8 - See above
    pub suggestedGear: Gear,                // i8 - Suggested gear for the player (1-8) 0 if no gear suggested
}

impl PacketCarTelemetry
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            header            : Header::unpack(bytes),

            carTelemetry      : Self::carTelemetry(&bytes[29..1349]),

            mfdFirstPlayer    : MFDPanel::from_u8(&bytes[1350]),
            mfdSecondaryPlayer: MFDPanel::from_u8(&bytes[1351]),
            suggestedGear     : Gear::from_u8_to_i8(&bytes[1352])
        }
    }

    pub fn carTelemetry(bytes: &[u8]) -> [CarTelemetry; 22]
    {
        let mut ct = [CarTelemetry::default(); 22];
        let size = size_of::<CarTelemetry>();

        for (i, ct) in ct.iter_mut().enumerate()
        {
            let s = size * i;
            let e = s + size;

            *ct = CarTelemetry::unpack(&bytes[s..e]);
        }

        ct
    }
}

/// # Car Status Packet
/// This packet details car statuses for all the cars in the race.
/// Frequency: Rate as specified in menus
/// Size: 1058 bytes
/// Version: 1
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum TC {
    Off = 0,
    Medium = 1,
    Full = 2,
    #[default]
    Poisoned = 255,
}

impl TC
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => TC::Off,
            1 => TC::Medium,
            2 => TC::Full,
            _ => TC::Poisoned,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum FuelMix {
    Lean = 0,
    Standard = 1,
    Rich = 2,
    Max = 3,
    #[default]
    Poisoned = 255,
}

impl FuelMix
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => FuelMix::Lean,
            1 => FuelMix::Standard,
            2 => FuelMix::Rich,
            3 => FuelMix::Max,
            _ => FuelMix::Poisoned,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum ActualCompound {
             C6 = 22,
             C0 = 21,
             C1 = 20,
             C2 = 19,
             C3 = 18,
             C4 = 17,
             C5 = 16,
          F2Wet = 15,
         F2Hard = 14,
       F2Medium = 13,
         F2Soft = 12,
    F2SuperSoft = 11,
     ClassicWet = 10,
     ClassicDry = 9,
            Wet = 8,
          Inter = 7,
        #[default]
       Poisoned = 255
}

impl ActualCompound {
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            22 => Self::C6,
            21 => Self::C0,
            20 => Self::C1,
            19 => Self::C2,
            18 => Self::C3,
            17 => Self::C4,
            16 => Self::C5,
            15 => Self::F2Wet,
            14 => Self::F2Hard,
            13 => Self::F2Medium,
            12 => Self::F2Soft,
            11 => Self::F2SuperSoft,
            10 => Self::ClassicWet,
             9 => Self::ClassicDry,
             8 => Self::Wet,
             7 => Self::Inter,
             _ => Self::Poisoned,
        }
    }
}

impl fmt::Display for ActualCompound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self
        {
            Self::C0            => write!(f, "C0"),
            Self::C1            => write!(f, "C1"),
            Self::C2            => write!(f, "C2"),
            Self::C3            => write!(f, "C3"),
            Self::C4            => write!(f, "C4"),
            Self::C5            => write!(f, "C5"),
            Self::C6            => write!(f, "C6"),
            Self::F2Wet         => write!(f, "w"),
            Self::F2Hard        => write!(f, "h"),
            Self::F2Medium      => write!(f, "m"),
            Self::F2Soft        => write!(f, "s"),
            Self::F2SuperSoft   => write!(f, "ss"),
            Self::ClassicWet    => write!(f, "w"),
            Self::ClassicDry    => write!(f, "d"),
            Self::Wet           => write!(f, "W"),
            Self::Inter         => write!(f, "I"),
            _                   => write!(f, ""),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum VisualCompound {
         OldHard = 22,
       OldMedium = 21,
         OldSoft = 20,
    OldSuperSoft = 19,
            Hard = 18,
          Medium = 17,
            Soft = 16,
          OldWet = 15,
      ClassicWet = 10,
      ClassicDry = 9,
             Wet = 8,
           Inter = 7,
         #[default]
        Poisoned = 0,
}

impl VisualCompound {
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            22 => Self::OldHard,
            21 => Self::OldMedium,
            20 => Self::OldSoft,
            19 => Self::OldSuperSoft,
            18 => Self::Hard,
            16 => Self::Soft,
            17 => Self::Medium,
            15 => Self::OldWet,
            10 => Self::ClassicWet,
             9 => Self::ClassicDry,
             8 => Self::Wet,
             7 => Self::Inter,
             _ => Self::Poisoned,
        }
    }
}

impl fmt::Display for VisualCompound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::OldHard      => write!(f, "h"),
            Self::OldMedium    => write!(f, "m"),
            Self::OldSoft      => write!(f, "s"),
            Self::OldSuperSoft => write!(f, "ss"),
            Self::Hard         => write!(f, "H"),
            Self::Medium       => write!(f, "M"),
            Self::Soft         => write!(f, "S"),
            Self::OldWet       => write!(f, "w"),
            Self::ClassicWet   => write!(f, "w"),
            Self::ClassicDry   => write!(f, "d"),
            Self::Wet          => write!(f, "W"),
            Self::Inter        => write!(f, "I"),
                             _ => write!(f, "?"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum ErsDeployMode {
    None = 0,
    Medium = 1,
    Hotlap = 2,
    Overtake = 3,
    #[default]
    Poisoned = 255,
}

impl ErsDeployMode
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => ErsDeployMode::None,
            1 => ErsDeployMode::Medium,
            2 => ErsDeployMode::Hotlap,
            3 => ErsDeployMode::Overtake,
            _ => ErsDeployMode::Poisoned,
        }
    }
}

#[repr(C, packed)] // Size: 44
#[derive(Debug, Default, Clone, Copy)]
pub struct CarStatus
{
    pub tractionControl: TC,            // u8
    pub antiLockBrakes: Assist,         // u8 - 0 (off) - 1 (on)
    pub fuelMix: FuelMix,               // u8
    pub frontBrakeBias: u8,             // Front brake bias (percentage)
    pub pitLimiterStatus: u8,           // Pit limiter status - 0 = off, 1 = on
    pub fuelInTank: f32,                // Current fuel mass
    pub fuelCapacity: f32,              // Fuel capacity
    pub fuelRemainingLaps: f32,         // Fuel remaining in terms of laps (value on MFD)
    pub maxRPM: u16,                    // Cars max RPM, point of rev limiter
    pub idleRPM: u16,                   // Cars idle RPM
    pub maxGears: u8,                   // Maximum number of gears
    pub drsAllowed: u8,                 // 0 = not allowed, 1 = allowed
    pub drsActivationDistance: u16,     // 0 = DRS not available, non-zero - DRS will be available in [X] metres
    pub actualTyre: ActualCompound,     // u8 - The Rubber on the Road
    pub visualTyre: VisualCompound,     // u8 - The Visual look of the Rubber
    pub tyresAgeLaps: u8,               // Age in laps of the current set of tyres
    pub vehicleFiaFlags: ZoneFlag,      // u8
    pub enginePowerICE: f32,            // Engine power output of ICE (W)
    pub enginePowerMGUK: f32,           // Engine power output of MGU-K (W)
    pub ersStoreEnergy: f32,            // ERS energy store in Joules
    pub ersDeployMode: ErsDeployMode,   // u8
    pub ersHarvestedThisLapMGUK: f32,   // ERS energy harvested this lap by MGU-K
    pub ersHarvestedThisLapMGUH: f32,   // ERS energy harvested this lap by MGU-H
    pub ersDeployedThisLap: f32,        // ERS energy deployed this lap
    pub networkPaused: u8,              // Whether the car is paused in a network game
}

impl CarStatus
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self
        {
                tractionControl:             TC::from_u8(&bytes[ 0]),
                 antiLockBrakes:         Assist::from_u8(&bytes[ 1]),
                        fuelMix:        FuelMix::from_u8(&bytes[ 2]),
                 frontBrakeBias: bytes[ 3],
               pitLimiterStatus: bytes[ 4],
                     fuelInTank:            f32::from_le_bytes([bytes[ 5], bytes[ 6], bytes[ 7], bytes[ 8]]),
                   fuelCapacity:            f32::from_le_bytes([bytes[ 9], bytes[10], bytes[11], bytes[12]]),
              fuelRemainingLaps:            f32::from_le_bytes([bytes[13], bytes[14], bytes[15], bytes[16]]),
                         maxRPM:            u16::from_le_bytes([bytes[17], bytes[18]]),
                        idleRPM:            u16::from_le_bytes([bytes[19], bytes[20]]),
                       maxGears: bytes[21],
                     drsAllowed: bytes[22],
          drsActivationDistance:            u16::from_le_bytes([bytes[23], bytes[24]]),
                     actualTyre: ActualCompound::from_u8(&bytes[25]),
                     visualTyre: VisualCompound::from_u8(&bytes[26]),
                   tyresAgeLaps: bytes[27],
                vehicleFiaFlags:       ZoneFlag::from_u8_to_i8(&bytes[28]),
                 enginePowerICE: f32::from_le_bytes([bytes[29], bytes[30], bytes[31], bytes[32]]),
                enginePowerMGUK: f32::from_le_bytes([bytes[33], bytes[34], bytes[35], bytes[36]]),           // Engine power output of MGU-K (W)
                 ersStoreEnergy:            f32::from_le_bytes([bytes[37], bytes[38], bytes[39], bytes[40]]),
                  ersDeployMode:  ErsDeployMode::from_u8(&bytes[41]),
        ersHarvestedThisLapMGUK:            f32::from_le_bytes([bytes[42], bytes[43], bytes[44], bytes[45]]),
        ersHarvestedThisLapMGUH:            f32::from_le_bytes([bytes[46], bytes[47], bytes[48], bytes[49]]),
             ersDeployedThisLap:            f32::from_le_bytes([bytes[50], bytes[51], bytes[52], bytes[53]]),
                  networkPaused: bytes[54],
        }
    }
}

#[repr(C, packed)] // Size: 1239
#[derive(Debug, Default, Clone, Copy)]
pub struct PacketCarStatus
{
    pub header: Header,                 // 24 Bytes - Header

    pub carStatus: [CarStatus; 22],
}

impl PacketCarStatus
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            header: Header::unpack(bytes),

            carStatus: Self::carStatus(&bytes[29..]),
        }
    }

    pub fn carStatus(bytes: &[u8]) -> [CarStatus; 22]
    {
        let mut cs = [CarStatus::default(); 22];
        let size = size_of::<CarStatus>();

        for (i, cs) in cs.iter_mut().enumerate()
        {
            let s = size * i;
            let e = s + size;

            *cs = CarStatus::unpack(&bytes[s..e]);
        }

        cs
    }
}

/// # Final Classification Packet
/// This packet details the final classification at the end of the race, and the data will match with the post race results screen. This is especially useful for multiplayer games where it is not always possible to send lap times on the final frame because of network delay.
/// Frequency: Once at the end of a race
/// Size: 1015 bytes
/// Version: 1
#[repr(C, packed)] // Size: 45 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct FinalClassification
{
    pub position: u8,               // Finishing position
    pub numLaps: u8,                // Number of laps completed
    pub gridPosition: u8,           // Grid position of `the car
    pub points: u8,                 // Number of points scored
    pub numPitStops: u8,            // Number of pit stops made
    pub resultStatus: ResultStatus, // u8
    pub bestLapTimeInMS: u32,       // Best lap time of the session in milliseconds
    pub totalRaceTime: f64,         // Total race time in seconds without penalties
    pub penaltiesTime: u8,          // Total penalties accumulated in seconds
    pub numPenalties: u8,           // Number of penalties applied to this driver
    pub numTyreStints: u8,          // Number of tyres stints up to maximum
    pub tyreStintsActual: [ActualCompound; 8], // u8x8 Actual tyres used by this driver
    pub tyreStintsVisual: [VisualCompound; 8], // u8x8 Visual tyres used by this driver
    pub tyreStintsEndLaps: [u8; 8], // The lap number stints end on
}

impl FinalClassification
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
                     position:                       bytes[ 0],
                      numLaps:                       bytes[ 1],
                 gridPosition:                       bytes[ 2],
                       points:                       bytes[ 3],
                  numPitStops:                       bytes[ 4],
                 resultStatus: ResultStatus::from_u8(bytes[ 5]),
              bestLapTimeInMS:   u32::from_le_bytes([bytes[ 6], bytes[ 7], bytes[ 8], bytes[ 9]]),
                totalRaceTime:   f64::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17]]),
                penaltiesTime:                       bytes[18],
                 numPenalties:                       bytes[19],
                numTyreStints:                       bytes[20],
             tyreStintsActual:     Self::actualTyre(&bytes[21..29]),
             tyreStintsVisual:     Self::visualTyre(&bytes[29..37]),
            tyreStintsEndLaps:                      [bytes[37], bytes[38], bytes[39], bytes[40], bytes[41], bytes[42], bytes[43], bytes[44]],
        }
    }

    pub fn actualTyre(bytes: &[u8]) -> [ActualCompound; 8]
    {
        let mut ac = [ActualCompound::default(); 8];

        for (i, ac) in ac.iter_mut().enumerate()
        {
            *ac = ActualCompound::from_u8(&bytes[i])
        }

        ac
    }
    fn visualTyre(bytes: &[u8]) -> [VisualCompound; 8]
    {
        let mut vc = [VisualCompound::default(); 8];

        for (i, vc) in vc.iter_mut().enumerate()
        {
            *vc = VisualCompound::from_u8(&bytes[i])
        }

        vc
    }
}

#[repr(C, packed)] // Size: 1014 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct PacketFinalClassification
{
    pub header: Header,             // 24 Bytes - Header

    pub numCars: u8,                // Number of cars in the final classification
    pub classificationData: [FinalClassification; 22],
}

impl PacketFinalClassification
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            header: Header::unpack(bytes),

            numCars: bytes[25],
            classificationData: Self::classificationData(&bytes[26..])
        }
    }

    pub fn classificationData(bytes: &[u8]) -> [FinalClassification; 22]
    {
        let mut fc = [FinalClassification::default(); 22];
        let size = size_of::<FinalClassification>();

        for (i, fc) in fc.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *fc = FinalClassification::unpack(&bytes[s..e]);
        }

        fc
    }
}

/// # Lobby Info Packet
/// This packet details the players currently in a multiplayer lobby. It details each player’s selected car, any AI involved in the game and also the ready status of each of the participants.
/// Frequency: Two every second when in the lobby
/// Size: 1191 bytes
/// Version: 1
#[repr(C, packed)] // Size: 53 Bytes
#[derive(Clone, Copy)]
pub struct LobbyInfo
{
    pub aiControlled: u8,       // Whether the vehicle is AI (1) or Human (0) controlled
    pub teamId: u8,             // Team id - see appendix (255 if no team currently selected)
    pub nationality: u8,        // Nationality of the driver
    pub name: [u8; 48],         // Name of participant in UTF-8 format – null terminated Will be truncated with ... (U+2026) if too long
    pub carNumber: u8,          // Car number of the player
    pub readyStatus: ReadyStatus,//u8
}

impl LobbyInfo
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            aiControlled: bytes[0],
            teamId      : bytes[1],
            nationality : bytes[2],
            name: match bytes[3..3+48].try_into()
                    {
                        Ok(str) => str,
                        Err(err) => {
                            dbg!(err);
                            [0; 48]
                        }
                    },
            carNumber   : bytes[51],
            readyStatus : ReadyStatus::from_u8(&bytes[52]),
        }
    }

    pub fn name_to_string(&self) -> String
    {
        std::str::from_utf8(&self.name).unwrap().trim_end_matches('\0').to_string()
    }
}

impl Default for LobbyInfo
{
    fn default() -> Self
    {
        Self {
            aiControlled: 0,
            teamId: 0,
            nationality: 0,
            name: [0; 48],
            carNumber: 0,
            readyStatus: ReadyStatus::default()
        }
    }
}

impl fmt::Debug for LobbyInfo
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LobbyInfo")
         .field("aiControlled", &self.aiControlled)
         .field(      "teamId", &self.teamId)
         .field( "nationality", &self.nationality)
         .field(        "name", &std::str::from_utf8(&self.name).unwrap().trim_end_matches('\0'))
         .field(   "carNumber", &self.carNumber)
         .field( "readyStatus", &self.readyStatus)
         .finish()
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum ReadyStatus {
    NotReady = 0,
    Ready = 1,
    Spectating = 2,
    #[default]
    Poisoned = 255,
}

impl ReadyStatus
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0 => ReadyStatus::NotReady,
            1 => ReadyStatus::Ready,
            2 => ReadyStatus::Spectating,
            _ => ReadyStatus::Poisoned,
        }
    }
}

#[repr(C, packed)] // Size: 1191 Bytes
#[derive(Debug, Clone, Copy)]
pub struct PacketLobbyInfo
{
    pub header: Header,             // 24 Bytes - Header

    pub numPlayers: u8,             // Number of players in the lobby data
    pub lobbyPlayers: [LobbyInfo; 22],
}

impl PacketLobbyInfo
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            header: Header::unpack(bytes),

            numPlayers: bytes[24],
            lobbyPlayers: Self::lobbyInfo(&bytes[25..]),
        }
    }

    pub fn lobbyInfo(bytes: &[u8]) -> [LobbyInfo; 22]
    {
        let mut li = [LobbyInfo::default(); 22];
        let size = size_of::<LobbyInfo>();

        for (i, li) in li.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *li = LobbyInfo::unpack(&bytes[s..e]);
        }

        li
    }
}

/// # Car Damage Packet
/// This packet details car damage parameters for all the cars in the race.
/// Frequency: 2 per second
/// Size: 948 bytes
/// Version: 1
#[repr(C, packed)] // Size: 42 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct CarDamage
{
    pub tyresWear: Wheels,          // Tyre wear (percentage)
    pub tyresDamage: [u8; 4],       // Tyre damage (percentage)
    pub brakesDamage: [u8; 4],      // Brakes damage (percentage)
    pub frontLeftWingDamage: u8,    // Front left wing damage (percentage)
    pub frontRightWingDamage: u8,   // Front right wing damage (percentage)
    pub rearWingDamage: u8,         // Rear wing damage (percentage)
    pub floorDamage: u8,            // Floor damage (percentage)
    pub diffuserDamage: u8,         // Diffuser damage (percentage)
    pub sidepodDamage: u8,          // Sidepod damage (percentage)
    pub drsFault: u8,               // Indicator for DRS fault, 0 = OK, 1 = fault
    pub ersFault: u8,               // Indicator for ERS fault, 0 = OK, 1 = fault
    pub gearBoxDamage: u8,          // Gear box damage (percentage)
    pub engineDamage: u8,           // Engine damage (percentage)
    pub engineMGUHWear: u8,         // Engine wear MGU-H (percentage)
    pub engineESWear: u8,           // Engine wear ES (percentage)
    pub engineCEWear: u8,           // Engine wear CE (percentage)
    pub engineICEWear: u8,          // Engine wear ICE (percentage)
    pub engineMGUKWear: u8,         // Engine wear MGU-K (percentage)
    pub engineTCWear: u8,           // Engine wear TC (percentage)
    pub engineBlown: u8,            // Engine blown, 0 = OK, 1 = fault
    pub engineSeized: u8,           // Engine seized, 0 = OK, 1 = fault
}

impl CarDamage
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
                       tyresWear: Wheels::unpack(&bytes[0..16]),
                     tyresDamage: [bytes[16], bytes[17], bytes[18], bytes[19]],
                    brakesDamage: [bytes[20], bytes[21], bytes[22], bytes[23]],
             frontLeftWingDamage: bytes[24],
            frontRightWingDamage: bytes[25],
                  rearWingDamage: bytes[26],
                     floorDamage: bytes[27],
                  diffuserDamage: bytes[28],
                   sidepodDamage: bytes[29],
                        drsFault: bytes[30],
                        ersFault: bytes[31],
                   gearBoxDamage: bytes[32],
                    engineDamage: bytes[33],
                  engineMGUHWear: bytes[34],
                    engineESWear: bytes[35],
                    engineCEWear: bytes[36],
                   engineICEWear: bytes[37],
                  engineMGUKWear: bytes[38],
                    engineTCWear: bytes[39],
                     engineBlown: bytes[40],
                    engineSeized: bytes[41],
        }
    }
}

#[repr(C, packed)] // Size: 948 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct PacketCarDamage
{
    pub header: Header,             // 24 Bytes - Header

    pub carDamageData: [CarDamage; 22],
}

impl PacketCarDamage
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            header: Header::unpack(bytes),

            carDamageData: Self::carDamage(&bytes[23..])
        }
    }

    pub fn carDamage(bytes: &[u8]) -> [CarDamage; 22]
    {
        let mut cd = [CarDamage::default(); 22];
        let size = size_of::<CarDamage>();

        for (i, cd) in cd.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *cd = CarDamage::unpack(&bytes[s..e]);
        }

        cd
    }
}

/// # Session History Packet
/// This packet contains lap times and tyre usage for the session. **This packet works slightly differently to other packets. To reduce CPU and bandwidth, each packet relates to a specific vehicle and is sent every 1/20 s, and the vehicle being sent is cycled through. Therefore in a 20 car race you should receive an update for each vehicle at least once per second.**
/// Note that at the end of the race, after the final classification packet has been sent, a final bulk update of all the session histories for the vehicles in that session will be sent.
/// Frequency: 20 per second but cycling through cars
/// Size: 1155 bytes
/// Version: 1
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Valid
{
    Lap         = 0b00000001,
    Sector1     = 0b00000010,
    Sector2     = 0b00000100,
    Sector2And1 = 0b00000110,
    Sector3     = 0b00001000,
    Sector3And1 = 0b00001010,
    Sector3And2 = 0b00001100,
    All         = 0b00001111,
    #[default]
    Poisoned    = 0b11111111,
}

impl Valid
{
    pub fn from_u8(byte: &u8) -> Self
    {
        match byte
        {
            0b00000001 => Valid::Lap,
            0b00000010 => Valid::Sector1,
            0b00000100 => Valid::Sector2,
            0b00000110 => Valid::Sector2And1,
            0b00001000 => Valid::Sector3,
            0b00001010 => Valid::Sector3And1,
            0b00001100 => Valid::Sector3And2,
            0b00001111 => Valid::All,
            _          => Valid::Poisoned,
        }
    }
}

#[repr(C, packed)] // Size: 11 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct LapHistory
{
    pub lapTimeInMS: u32,               // u32 Lap time in milliseconds
    pub sector1TimeInMS: u16,           // u16 Sector 1 time in milliseconds
    pub sector2TimeInMS: u16,           // u16 Sector 2 time in milliseconds
    pub sector3TimeInMS: u16,           // u16 Sector 3 time in milliseconds
    pub lapValidBitFlags: Valid,        // u8 - 0x01 bit set-lap valid, 0x02 bit set-sector 1 valid 0x04 bit set-sector 2 valid, 0x08 bit set-sector 3 valid
}

impl LapHistory
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            lapTimeInMS     : u32::from_le_bytes([bytes[ 0], bytes[ 1], bytes[ 2], bytes[ 3]]),
            sector1TimeInMS : u16::from_le_bytes([bytes[ 4], bytes[ 5]]),
            sector2TimeInMS : u16::from_le_bytes([bytes[ 6], bytes[ 8]]),
            sector3TimeInMS : u16::from_le_bytes([bytes[ 8], bytes[10]]),
            lapValidBitFlags: Valid::from_u8(&bytes[10])
        }
    }
}

#[repr(C, packed)] // Size: 3 Bytes
#[derive(Debug, Default, Clone, Copy)]
pub struct TyreStintHistory
{
    pub endLap: u8,             // Lap the tyre usage ends on (255 of current tyre)
    pub tyreActualCompound: ActualCompound, // u8 Actual tyres used by this driver
    pub tyreVisualCompound: VisualCompound, // u8 Visual tyres used by this driver
}

impl TyreStintHistory
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            endLap: bytes[0],
            tyreActualCompound: ActualCompound::from_u8(&bytes[1]),
            tyreVisualCompound: VisualCompound::from_u8(&bytes[2]),
        }
    }
}

#[repr(C, packed)] // Size: 1155 Bytes
#[derive(Debug, Clone, Copy)]
pub struct PacketSessionHistory
{
    pub header: Header,         // 24 Bytes - Header

    pub carIdx: u8,             // Index of the car this lap data relates to
    pub numLaps: u8,            // Num laps in the data (including current partial lap)
    pub numTyreStints: u8,      // Number of tyre stints in the data

    pub bestLapTimeLapNum: u8,  // Lap the best lap time was achieved on
    pub bestSector1LapNum: u8,  // Lap the best Sector 1 time was achieved on
    pub bestSector2LapNum: u8,  // Lap the best Sector 2 time was achieved on
    pub bestSector3LapNum: u8,  // Lap the best Sector 3 time was achieved on

    pub lapHistory: [LapHistory; 100], // 11 Bytes * 100 - 100 laps of data max
    pub tyreStintsHistory: [TyreStintHistory; 8], // 3 Bytes * 8
}

impl PacketSessionHistory
{
    pub fn unpack(bytes: &[u8]) -> Self
    {
        Self {
            header           : Header::unpack(bytes),

            carIdx           : bytes[24],
            numLaps          : bytes[25],
            numTyreStints    : bytes[26],
            bestLapTimeLapNum: bytes[27],
            bestSector1LapNum: bytes[28],
            bestSector2LapNum: bytes[29],
            bestSector3LapNum: bytes[30],
            lapHistory       : Self::lapHistory(&bytes[24+7..(24+7)+(11*100)]),
            tyreStintsHistory: Self::tyreStintHistory(&bytes[(24+7)+(11*100)..(24+7)+(11*100)+(3*8)]),
        }
    }

    pub fn lapHistory(bytes: &[u8]) -> [LapHistory; 100]
    {
        let mut lh = [LapHistory::default(); 100];
        let size = 11;

        for (i, lh) in lh.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *lh = LapHistory::unpack(&bytes[s..e]);
        }

        lh
    }

    pub fn tyreStintHistory(bytes: &[u8]) -> [TyreStintHistory; 8]
    {
        let mut tsh = [TyreStintHistory::default(); 8];
        let size = 3;

        for (i, tsh) in tsh.iter_mut().enumerate()
        {
            let s = i * size;
            let e = s + size;

            *tsh = TyreStintHistory::unpack(&bytes[s..e]);
        }

        tsh
    }
}
