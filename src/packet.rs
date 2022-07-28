#![allow(non_snake_case)]

// https://answers.ea.com/t5/General-Discussion/F1-22-UDP-Specification/td-p/11551274

/**
 * # Packet Header
 * Each packet has the following header:
 */

#[derive(Debug, Clone, Copy)]
pub struct Header
{
    pub packetFormat: u16,              // 2022
    pub gameMajorVersion: u8,           // Game major version - "X.00"
    pub gameMinorVersion: u8,           // Game minor version - "1.XX"
    pub packetVersion: u8,              // Version of this packet type, all start from 1
    pub packetId: PacketId,             // Identifier for the packet type, see below
    pub sessionUID: u64,                // Unique identifier for the session
    pub sessionTime: f32,               // Session timestamp
    pub frameIdentifier: u32,           // Identifier for the frame the data was retrieved on
    pub playerCarIndex: u8,             // Index of player's car in the array
    pub secondaryPlayerCarIndex: u8,    // Index of secondary player's car in the array (splitscreen) 255 if no second player
}

/**
 * # Packet IDs
 * The packets IDs are as follows
 */
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PacketId {
    Motion = 0,                         // Contains all motion data for player’s car – only sent while player is in control
    Session = 1,                        // Data about the session – track, time left
    LapData = 2,                        // Data about all the lap times of cars in the session
    Event = 3,                          // Various notable events that happen during a session
    Participants = 4,                   // List of participants in the session, mostly relevant for multiplayer
    CarSetups = 5,                      // Packet detailing car setups for cars in the race
    CarTelemetry = 6,                   // Telemetry data for all cars
    CarStatus = 7,                      // Status data for all cars
    FinalClassification = 8,            // Final classification confirmation at the end of a race
    LobbyInfo = 9,                      // Information about players in a multiplayer lobby
    CarDamage = 10,                     // Damage status for all cars
    SessionHistory = 11                 // Lap and tyre data for session
}

/**
 * # Motion Packet
 * The motion packet gives physics data for all the cars being driven. There is additional data for the car being driven with the goal of being able to drive a motion platform setup.
 * N.B. For the normalised vectors below, to convert to float values divide by 32767.0f – 16-bit signed values are used to pack the data and on the assumption that direction values are always between -1.0f and 1.0f.
 * Frequency: Rate as specified in menus
 * Size: 1464 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct CarMotion
{
    pub worldPositionX: f32,                // World space X position
    pub worldPositionY: f32,                // World space Y position
    pub worldPositionZ: f32,                // World space Z position
    pub worldVelocityX: f32,                // Velocity in world space X
    pub worldVelocityY: f32,                // Velocity in world space Y
    pub worldVelocityZ: f32,                // Velocity in world space Z
    pub worldForwardDirX: i16,              // World space forward X direction (normalised)
    pub worldForwardDirY: i16,              // World space forward Y direction (normalised)
    pub worldForwardDirZ: i16,              // World space forward Z direction (normalised)
    pub worldRightDirX: i16,                // World space right X direction (normalised)
    pub worldRightDirY: i16,                // World space right Y direction (normalised)
    pub worldRightDirZ: i16,                // World space right Z direction (normalised)
    pub gForceLateral: f32,                 // Lateral G-Force component
    pub gForceLongitudinal: f32,            // Longitudinal G-Force component
    pub gForceVertical: f32,                // Vertical G-Force component
    pub yaw: f32,                           // Yaw angle in radians
    pub pitch: f32,                         // Pitch angle in radians
    pub roll: f32,                          // Roll angle in radians
}

#[derive(Debug, Clone, Copy)]
pub struct PacketMotion
{
    pub header: Header,               // Header

    pub carMotion: [CarMotion; 22],   // Data for all cars on track

    // Extra player car ONLY data
    pub suspensionPosition: [f32; 4],        // Note: All wheel arrays have the following order:
    pub suspensionVelocity: [f32; 4],        // RL, RR, FL, FR
    pub suspensionAcceleration: [f32; 4],    // RL, RR, FL, FR
    pub wheelSpeed: [f32; 4],                // Speed of each wheel
    pub wheelSlip: [f32; 4],                 // Slip ratio for each wheel
    pub localVelocityX: f32,               // Velocity in local space
    pub localVelocityY: f32,               // Velocity in local space
    pub localVelocityZ: f32,               // Velocity in local space
    pub angularVelocityX: f32,             // Angular velocity x-component
    pub angularVelocityY: f32,             // Angular velocity y-component
    pub angularVelocityZ: f32,             // Angular velocity z-component
    pub angularAccelerationX: f32,         // Angular velocity x-component
    pub angularAccelerationY: f32,         // Angular velocity y-component
    pub angularAccelerationZ: f32,         // Angular velocity z-component
    pub frontWheelsAngle: f32,             // Current front wheels angle in radians
}

/**
 * # Session Packet
 * The session packet includes details about the current session in progress.
 * Frequency: 2 per second
 * Size: 632 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct MarshalZone
{
    pub zoneStart: f32,                // Fraction (0..1) of way through the lap the marshal zone starts
    pub zoneFlag: ZoneFlag,
}

#[repr(i8)]
#[derive(Debug, Clone, Copy)]
pub enum ZoneFlag {
    Invalid = -1,
    None = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
    Red = 4,
}

#[derive(Debug, Clone, Copy)]
pub struct WeatherForecast
{
    pub sessionType: Session,
    pub timeOffset: u8,               // Time in minutes the forecast is for
    pub weather: Weather,
    pub trackTemperature: i8,         // Track temp. in degrees Celsius
    pub trackTemperatureChange: Temperature,
    pub airTemperature: i8,           // Air temp. in degrees celsius
    pub airTemperatureChange: Temperature,
    pub rainPercentage: u8,           // Rain percentage (0-100)
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Weather {
    Clear = 0,
    LightCloud = 1,
    Overcast = 2,
    RainLight = 3,
    RainHeavy = 4,
    RainStorm = 5
}

#[repr(i8)]
#[derive(Debug, Clone, Copy)]
pub enum Temperature {
    Up = 0,
    Down = 1,
    None = 2
}

#[derive(Debug, Clone, Copy)]
pub struct PacketSession
{
    pub header: Header,           // Header

    pub weather: Weather,
    pub trackTemperature: i8,           // Track temp. in degrees celsius
    pub airTemperature: i8,             // Air temp. in degrees celsius
    pub totalLaps: u8,                  // Total number of laps in this race
    pub trackLength: u16,               // Track length in metres
    pub sessionType: Session,
    pub trackId: i8,                    // -1 for unknown, see appendix
    pub formula: Formula,
    pub sessionTimeLeft: u16,           // Time left in session in seconds
    pub sessionDuration: u16,           // Session duration in seconds
    pub pitSpeedLimit: u8,              // Pit speed limit in kilometres per hour
    pub gamePaused: u8,                 // Whether the game is paused – network game only
    pub isSpectating: u8,               // Whether the player is spectating
    pub spectatorCarIndex: u8,          // Index of the car being spectated
    pub sliProNativeSupport: u8,        // SLI Pro support, 0 = inactive, 1 = active
    pub numMarshalZones: u8,            // Number of marshal zones to follow
    pub marshalZones: [MarshalZone; 21],  // List of marshal zones – max 21
    pub safetyCarStatus: SafetyCar,
    pub networkGame: u8,                // 0 = offline, 1 = online
    pub numWeatherForecasts: u8,  // Number of weather samples to follow
    pub weatherForecastSamples: [WeatherForecast; 56], // Array of weather forecast samples
    pub forecastAccuracy: u8,           // 0 = Perfect, 1 = Approximate
    pub aiDifficulty: u8,               // AI Difficulty rating – 0-110
    pub seasonLinkIdentifier: u32,      // Identifier for season - persists across saves
    pub weekendLinkIdentifier: u32,     // Identifier for weekend - persists across saves
    pub sessionLinkIdentifier: u32,     // Identifier for session - persists across saves
    pub pitStopWindowIdealLap: u8,      // Ideal lap to pit on for current strategy (player)
    pub pitStopWindowLatestLap: u8,     // Latest lap to pit on for current strategy (player)
    pub pitStopRejoinPosition: u8,      // Predicted position to rejoin at (player)
    pub steeringAssist: u8,             // 0 = off, 1 = on
    pub brakingAssist: u8,              // 0 = off, 1 = low, 2 = medium, 3 = high
    pub gearboxAssist: u8,              // 1 = manual, 2 = manual & suggested gear, 3 = auto
    pub pitAssist: u8,                  // 0 = off, 1 = on
    pub pitReleaseAssist: u8,           // 0 = off, 1 = on
    pub ERSAssist: u8,                  // 0 = off, 1 = on
    pub DRSAssist: u8,                  // 0 = off, 1 = on
    pub dynamicRacingLine: u8,          // 0 = off, 1 = corners only, 2 = full
    pub dynamicRacingLineType: u8,      // 0 = 2D, 1 = 3D
    pub gameMode: u8,                   // Game mode id - see appendix
    pub ruleSet: u8,                    // Ruleset - see appendix
    pub timeOfDay: u32,                 // Local time of day - minutes since midnight
    pub sessionLength: SessionLength,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Formula {
    Modern = 0,
    Classic = 1,
    Formula2 = 2,
    Generic = 3,
    Beta = 4,
    Supercars = 5,
    Esports = 6,
    Formula22021 = 7
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SafetyCar {
    Ready = 0,
    Deployed = 1,
    Virtual = 2,
    FormationLap = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SessionLength {
     None = 0,
     VeryShort = 2,
     Short = 3,
     Medium = 4,
     MediumLong = 5,
     Long = 6,
     Full = 7
}

/**
 * # Lap Data Packet
 * The lap data packet gives details of all the cars in the session.
 * Frequency: Rate as specified in menus
 * Size: 972 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct Lap
{
    pub lastLapTimeInMS: u32,               // Last lap time in milliseconds
    pub currentLapTimeInMS: u32,            // Current time around the lap in milliseconds
    pub sector1TimeInMS: u16,               // Sector 1 time in milliseconds
    pub sector2TimeInMS: u16,               // Sector 2 time in milliseconds
    pub lapDistance: f32,                   // Distance vehicle is around current lap in metres – could be negative if line hasn’t been crossed yet
    pub totalDistance: f32,                 // Total distance travelled in session in metres – could be negative if line hasn’t been crossed yet
    pub safetyCarDelta: f32,                // Delta in seconds for safety car
    pub carPosition: u8,                    // Car race position
    pub currentLapNum: u8,                  // Current lap number
    pub pitStatus: PitStatus,
    pub numPitStops: u8,                    // Number of pit stops taken in this race
    pub sector: u8,                         // 0 = sector1, 1 = sector2, 2 = sector3
    pub currentLapInvalid: u8,              // Current lap invalid - 0 = valid, 1 = invalid
    pub penalties: u8,                      // Accumulated time penalties in seconds to be added
    pub warnings: u8,                       // Accumulated number of warnings issued
    pub numUnservedDriveThroughPens: u8,    // Num drive through pens left to serve
    pub numUnservedStopGoPens: u8,          // Num stop go pens left to serve
    pub gridPosition: u8,                   // Grid position the vehicle started the race in
    pub driverStatus: Driver,
    pub resultStatus: ResultStatus,
    pub pitLaneTimerActive: u8,             // Pit lane timing, 0 = inactive, 1 = active
    pub pitLaneTimeInLaneInMS: u16,         // If active, the current time spent in the pit lane in ms
    pub pitStopTimerInMS: u16,              // Time of the actual pit stop in ms
    pub pitStopShouldServePen: u8,          // Whether the car should serve a penalty at this stop
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PitStatus {
    None = 0,
    Pitting = 1,
    InPitArea = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Driver {
    InGarage = 0,
    OnFlyingLap = 1,
    InLap = 2,
    OutLap = 3,
    OnTrack = 4,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ResultStatus {
    Invalid = 0,
    Inactive = 1,
    Active = 2,
    Finished = 3,
    DidNotFinish = 4,
    Disqualified = 5,
    NotClassified = 6,
    Retired = 7,
}

#[derive(Debug, Clone, Copy)]
pub struct PacketLap
{
    pub header: Header,                     // Header

    pub laps: [Lap; 22],                    // Lap data for allpub  cars on track

    pub timeTrialPBCarIdx: u8,              // Index of Personal Best car in time trial (255 if invalid)
    pub timeTrialRivalCarIdx: u8,           // Index of Rival car in time trial (255 if invalid)
}

/**
 * # Event Packet
 * This packet gives details of events that happen during the course of a session.
 * Frequency: When the event occurs
 * Size: 40 bytes
 * Version: 1
 */

// The event details packet is different for each type of event.
// Make sure only the correct type is interpreted.
#[derive(Clone, Copy)]
union EventDetails
{
    pub fastestLap: FastestLap,
    pub retirement: Retirement,
    pub teamMateInPits: TeamMateInPits,
    pub raceWinner: RaceWinner,
    pub penalty: Penalty,
    pub speedTrap: SpeedTrap,
    pub startLights: StartLights,
    pub driveThroughPenaltyServed: DriveThroughPenaltyServed,
    pub stopGoPenaltyServed: StopGoPenaltyServed,
    pub flashback: Flashback,
    pub buttons: Buttons,
}

#[derive(Debug, Clone, Copy)]
pub struct FastestLap
{
    pub vehicleIdx: u8,                 // Vehicle index of car achieving fastest lap
    pub lapTime: f32,                   // Lap time is in seconds
}

#[derive(Debug, Clone, Copy)]
pub struct Retirement
{
    pub vehicleIdx: u8,                 // Vehicle index of car retiring
}

#[derive(Debug, Clone, Copy)]
pub struct TeamMateInPits
{
    pub vehicleIdx: u8,                 // Vehicle index of team mate
}

#[derive(Debug, Clone, Copy)]
pub struct RaceWinner
{
    pub vehicleIdx: u8,                 // Vehicle index of the race winner
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct SpeedTrap
{
    pub vehicleIdx: u8,                 // Vehicle index of the vehicle triggering speed trap
    pub speed: f32,                     // Top speed achieved in kilometres per hour
    pub isOverallFastestInSession: u8,  // Overall fastest speed in session = 1, otherwise 0
    pub isDriverFastestInSession: u8,   // Fastest speed for driver in session = 1, otherwise 0
    pub fastestVehicleIdxInSession: u8, // Vehicle index of the vehicle that is the fastest in this session
    pub fastestSpeedInSession: f32,     // Speed of the vehicle that is the fastest in this session
}

#[derive(Debug, Clone, Copy)]
pub struct StartLights
{
    pub numLights: u8,                  // Number of lights showing
}

#[derive(Debug, Clone, Copy)]
pub struct DriveThroughPenaltyServed
{
    pub vehicleIdx: u8,                 // Vehicle index of the vehicle serving drive through
}

#[derive(Debug, Clone, Copy)]
pub struct StopGoPenaltyServed
{
    pub vehicleIdx: u8,                 // Vehicle index of the vehicle serving stop go
}

#[derive(Debug, Clone, Copy)]
pub struct Flashback
{
    pub flashbackFrameIdentifier: u32,  // Frame identifier flashed back to
    pub flashbackSessionTime: f32,      // Session time flashed back to
}

#[derive(Debug, Clone, Copy)]
pub struct Buttons
{
    pub buttonStatus: u32,              // Bit flags specifying which buttons are being pressed currently - see appendices
}

#[derive(Clone, Copy)]
pub struct PacketEvent
{
    pub header: Header,                     // Header

    pub eventStringCode: EventStringCode,   // Event string code, see below
    pub eventDetails: EventDetails,         // Event details - should be interpreted differently for each type
}

/**
 * # Event String Codes
 */
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum EventStringCode
{
    SessionStarted = 0x83838465,            // "SSTA" Sent when the session starts
    SessionEnded = 0x83697868,              // "SEND" Sent when the session ends
    FastestLap = 0x70857680,                // "FTLP" When a driver achieves the fastest lap
    Retirement = 0x82847784,                // "RTMT" When a driver retires
    DRSenabled = 0x68828369,                // "DRSE" Race control have enabled DRS
    DRSdisabled = 0x68828368,               // "DRSD" Race control have disabled DRS
    TeamMateInPits = 0x84778084,            // "TMPT" Your team mate has entered the pits
    ChequeredFlag = 0x67728170,             // "CHQF" The chequered flag has been waved
    RaceWinner = 0x82678778,                // "RCWN" The race winner is announced
    Penalty = 0x80697865,                   // "PENA" A penalty has been issued – details in event
    SpeedTrap = 0x83808480,                 // "SPTP" Speed trap has been triggered by fastest speed
    StartLights = 0x83847671,               // "STLG" Start lights – number shown
    LightsOut = 0x76717984,                 // "LGOT" Lights out
    DriveThroughPenaltyServed = 0x68848386, // "DTSV" Drive through penalty served
    StopGoPenaltyServed = 0x83717386,       // "SGSV" Stop go penalty served
    Flashback = 0x70766675,                 // "FLBK" Flashback activated
    Buttons = 0x66858478,                   // "BUTN" Button status changed
}

/**
 * # Participants Packet
 * This is a list of participants in the race. If the vehicle is controlled by AI, then the name will be the driver name. If this is a multiplayer game, the names will be the Steam Id on PC, or the LAN name if appropriate.
 * N.B. on Xbox One, the names will always be the driver name, on PS4 the name will be the LAN name if playing a LAN game, otherwise it will be the driver name. 
 * The array should be indexed by vehicle index.
 * Frequency: Every 5 seconds
 * Size: 1257 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct Participant<'a>
{
    pub aiControlled: u8,       // Whether the vehicle is AI (1) or Human (0) controlled
    pub driverId: u8,           // Driver id - see appendix, 255 if network human
    pub networkId: u8,          // Network id – unique identifier for network players
    pub teamId: u8,             // Team id - see appendix
    pub myTeam: u8,             // My team flag – 1 = My Team, 0 = otherwise
    pub raceNumber: u8,         // Race number of the car
    pub nationality: u8,        // Nationality of the driver
    pub name: [&'a str; 48],    // Name of participant in UTF-8 format – null terminated Will be truncated with … (U+2026) if too long
    pub yourTelemetry: u8,      // The player's UDP setting, 0 = restricted, 1 = public
}

#[derive(Debug, Clone, Copy)]
pub struct PacketParticipantsData<'a>
{
    pub header: Header,   // Header

    pub numActiveCars: u8,      // Number of active cars in the data – should match number of cars on HUD
    pub participants: [Participant<'a>; 22],
}

/**
 * # Car Setups Packet
 * This packet details the car setups for each vehicle in the session. Note that in multiplayer games, other player cars will appear as blank, you will only be able to see your car setup and AI cars.
 * Frequency: 2 per second
 * Size: 1102 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
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
    pub rearLeftTyrePressure: f32,      // Rear left tyre pressure (PSI)
    pub rearRightTyrePressure: f32,     // Rear right tyre pressure (PSI)
    pub frontLeftTyrePressure: f32,     // Front left tyre pressure (PSI)
    pub frontRightTyrePressure: f32,    // Front right tyre pressure (PSI)
    pub ballast: u8,                    // Ballast
    pub fuelLoad: f32,                  // Fuel load
}

#[derive(Debug, Clone, Copy)]
pub struct PacketCarSetup
{
    pub header: Header,            // Header

    pub carSetups: [CarSetup; 22],
}

/**
 * # Car Telemetry Packet
 * This packet details telemetry for all the cars in the race. It details various values that would be recorded on the car such as speed, throttle application, DRS etc. Note that the rev light configurations are presented separately as well and will mimic real life driver preferences.
 * Frequency: Rate as specified in menus
 * Size: 1347 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct CarTelemetry
{
    pub speed: u16,                         // Speed of car in kilometres per hour
    pub throttle: f32,                      // Amount of throttle applied (0.0 to 1.0)
    pub steer: f32,                         // Steering (-1.0 (full lock left) to 1.0 (full lock right))
    pub brake: f32,                         // Amount of brake applied (0.0 to 1.0)
    pub clutch: u8,                         // Amount of clutch applied (0 to 100)
    pub gear: i8,                           // Gear selected (1-8, N=0, R=-1)
    pub engineRPM: u16,                     // Engine RPM
    pub drs: u8,                            // 0 = off, 1 = on
    pub revLightsPercent: u8,               // Rev lights indicator (percentage)
    pub revLightsBitValue: u16,             // Rev lights (bit 0 = leftmost LED, bit 14 = rightmost LED)
    pub brakesTemperature: [u16; 4],          // Brakes temperature (celsius)
    pub tyresSurfaceTemperature: [u8; 4],     // Tyres surface temperature (celsius)
    pub tyresInnerTemperature: [u8; 4],       // Tyres inner temperature (celsius)
    pub engineTemperature: u16,             // Engine temperature (celsius)
    pub tyresPressure: [f32; 4],              // Tyres pressure (PSI)
    pub surfaceType: [u8; 4],                 // Driving surface, see appendices
}

#[derive(Debug, Clone, Copy)]
pub struct PacketCarTelemetry
{
    pub header: Header,               // Header

    pub carTelemetryData: [CarTelemetry; 22],

    pub mfdPanelIndex: u8,                  // Index of MFD panel open - 255 = MFD closed Single player, race – 0 = Car setup, 1 = Pits 2 = Damage, 3 =  Engine, 4 = Temperatures May vary depending on game mode
    pub mfdPanelIndexSecondaryPlayer: u8,   // See above
    pub suggestedGear: i8,                  // Suggested gear for the player (1-8) 0 if no gear suggested
}

/**
 * # Car Status Packet
 * This packet details car statuses for all the cars in the race.
 * Frequency: Rate as specified in menus
 * Size: 1058 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct CarStatus
{
    pub tractionControl: TractionControl,
    pub antiLockBrakes: u8,             // 0 (off) - 1 (on)
    pub fuelMix: FuelMix,
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
    pub actualTyreCompound: u8,         // F1 Modern - 16 = C5, 17 = C4, 18 = C3, 19 = C2, 20 = C1 7 = inter, 8 = wet
                                        // F1 Classic - 9 = dry, 10 = wet
                                        // F2 – 11 = super soft, 12 = soft, 13 = medium, 14 = hard 15 = wet
    pub visualTyreCompound: u8,         // F1 visual (can be different from actual compound)
                                        // 16 = soft, 17 = medium, 18 = hard, 7 = inter, 8 = wet
                                        // F1 Classic – same as above
                                        // F2 ‘19, 15 = wet, 19 – super soft, 20 = soft
                                        // 21 = medium , 22 = hard
    pub tyresAgeLaps: u8,               // Age in laps of the current set of tyres
    pub vehicleFiaFlags: ZoneFlag,
    pub ersStoreEnergy: f32,            // ERS energy store in Joules
    pub ersDeployMode: ErsDeployMode,
    pub ersHarvestedThisLapMGUK: f32,   // ERS energy harvested this lap by MGU-K
    pub ersHarvestedThisLapMGUH: f32,   // ERS energy harvested this lap by MGU-H
    pub ersDeployedThisLap: f32,        // ERS energy deployed this lap
    pub networkPaused: u8,              // Whether the car is paused in a network game
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TractionControl {
    Off = 0,
    Medium = 1,
    Full = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FuelMix {
    Lean = 0,
    Standard = 1,
    Rich = 2,
    Max = 3,
}

// m_visualTyreCompound is a todo.
// m_visualTyreCompound is a todo.

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ErsDeployMode {
    None = 0,
    Medium = 1,
    Hotlap = 2,
    Overtake = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct PacketCarStatus
{
    pub header: Header,           // Header

    pub carStatusData: [CarStatus; 22],
}

/**
 * # Final Classification Packet
 * This packet details the final classification at the end of the race, and the data will match with the post race results screen. This is especially useful for multiplayer games where it is not always possible to send lap times on the final frame because of network delay.
 * Frequency: Once at the end of a race
 * Size: 1015 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct FinalClassification
{
    pub position: u8,               // Finishing position
    pub numLaps: u8,                // Number of laps completed
    pub gridPosition: u8,           // Grid position of the car
    pub points: u8,                 // Number of points scored
    pub numPitStops: u8,            // Number of pit stops made
    pub resultStatus: ResultStatus,
    pub bestLapTimeInMS: u32,       // Best lap time of the session in milliseconds
    pub totalRaceTime: f64,         // Total race time in seconds without penalties
    pub penaltiesTime: u8,          // Total penalties accumulated in seconds
    pub numPenalties: u8,           // Number of penalties applied to this driver
    pub numTyreStints: u8,          // Number of tyres stints up to maximum
    pub tyreStintsActual: [u8; 8],  // Actual tyres used by this driver
    pub tyreStintsVisual: [u8; 8],  // Visual tyres used by this driver
    pub tyreStintsEndLaps: [u8; 8], // The lap number stints end on
}

#[derive(Debug, Clone, Copy)]
pub struct PacketFinalClassification
{
    pub header: Header,             // Header

    pub numCars: u8,                // Number of cars in the final classification
    pub classificationData: [FinalClassification; 22],
}

/**
 * # Lobby Info Packet
 * This packet details the players currently in a multiplayer lobby. It details each player’s selected car, any AI involved in the game and also the ready status of each of the participants.
 * Frequency: Two every second when in the lobby
 * Size: 1191 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct LobbyInfo<'a>
{
    pub aiControlled: u8,       // Whether the vehicle is AI (1) or Human (0) controlled
    pub teamId: u8,             // Team id - see appendix (255 if no team currently selected)
    pub nationality: u8,        // Nationality of the driver
    pub name: [&'a str; 48],    // Name of participant in UTF-8 format – null terminated Will be truncated with ... (U+2026) if too long
    pub carNumber: u8,          // Car number of the player
    pub readyStatus: ReadyStatus,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ReadyStatus {
    NotReady = 0,
    Ready = 1,
    Spectating = 2,
}

#[derive(Debug, Clone, Copy)]
pub struct PacketLobbyInfo<'a>
{
    pub header: Header,   // Header

    pub numPlayers: u8,         // Number of players in the lobby data
    pub lobbyPlayers: [LobbyInfo<'a>; 22],
}


/**
 * # Car Damage Packet
 * This packet details car damage parameters for all the cars in the race.
 * Frequency: 2 per second
 * Size: 948 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct CarDamage
{
    pub tyresWear: [f32; 4],        // Tyre wear (percentage)
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

#[derive(Debug, Clone, Copy)]
pub struct PacketCarDamage
{
    pub header: Header,       // Header

    pub carDamageData: [CarDamage; 22],
}

/**
 * # Session History Packet
 * This packet contains lap times and tyre usage for the session. **This packet works slightly differently to other packets. To reduce CPU and bandwidth, each packet relates to a specific vehicle and is sent every 1/20 s, and the vehicle being sent is cycled through. Therefore in a 20 car race you should receive an update for each vehicle at least once per second.**
 * Note that at the end of the race, after the final classification packet has been sent, a final bulk update of all the session histories for the vehicles in that session will be sent.
 * Frequency: 20 per second but cycling through cars
 * Size: 1155 bytes
 * Version: 1
 */
#[derive(Debug, Clone, Copy)]
pub struct LapHistory
{
    pub lapTimeInMS: u32,       // Lap time in milliseconds
    pub sector1TimeInMS: u16,   // Sector 1 time in milliseconds
    pub sector2TimeInMS: u16,   // Sector 2 time in milliseconds
    pub sector3TimeInMS: u16,   // Sector 3 time in milliseconds
    pub lapValidBitFlags: u8,   // 0x01 bit set-lap valid, 0x02 bit set-sector 1 valid 0x04 bit set-sector 2 valid, 0x08 bit set-sector 3 valid
}

#[derive(Debug, Clone, Copy)]
pub struct TyreStintHistory
{
    pub endLap: u8,             // Lap the tyre usage ends on (255 of current tyre)
    pub tyreActualCompound: u8, // Actual tyres used by this driver
    pub tyreVisualCompound: u8, // Visual tyres used by this driver
}

#[derive(Debug, Clone, Copy)]
pub struct PacketSessionHistoryData
{
    pub header: Header,   // Header

    pub carIdx: u8,             // Index of the car this lap data relates to
    pub numLaps: u8,            // Num laps in the data (including current partial lap)
    pub numTyreStints: u8,      // Number of tyre stints in the data

    pub bestLapTimeLapNum: u8,  // Lap the best lap time was achieved on
    pub bestSector1LapNum: u8,  // Lap the best Sector 1 time was achieved on
    pub bestSector2LapNum: u8,  // Lap the best Sector 2 time was achieved on
    pub bestSector3LapNum: u8,  // Lap the best Sector 3 time was achieved on

    pub lapHistory: [LapHistory; 100], // 100 laps of data max
    pub tyreStintsHistory: [TyreStintHistory; 8],
}
