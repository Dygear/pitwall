// https://answers.ea.com/t5/General-Discussion/F1-22-UDP-Specification/td-p/11551274

/**
 * # Packet Header
 * Each packet has the following header:
 */

pub struct PacketHeader
{
    pub u16 m_packetFormat;             // 2022
    pub u8  m_gameMajorVersion;         // Game major version - "X.00"
    pub u8  m_gameMinorVersion;         // Game minor version - "1.XX"
    pub u8  m_packetVersion;            // Version of this packet type, all start from 1
    pub u8  m_packetId;                 // Identifier for the packet type, see below
    pub u64 m_sessionUID;               // Unique identifier for the session
    pub f32 m_sessionTime;              // Session timestamp
    pub u32 m_frameIdentifier;          // Identifier for the frame the data was retrieved on
    pub u8  m_playerCarIndex;           // Index of player's car in the array
    pub u8  m_secondaryPlayerCarIndex;  // Index of secondary player's car in the array (splitscreen) 255 if no second player
};

/**
 * # Packet IDs
 * The packets IDs are as follows
 */
#[repr(u8)]
pub enum m_packetId: u8 {
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
};

/**
 * # Motion Packet
 * The motion packet gives physics data for all the cars being driven. There is additional data for the car being driven with the goal of being able to drive a motion platform setup.
 * N.B. For the normalised vectors below, to convert to float values divide by 32767.0f – 16-bit signed values are used to pack the data and on the assumption that direction values are always between -1.0f and 1.0f.
 * Frequency: Rate as specified in menus
 * Size: 1464 bytes
 * Version: 1
 */
pub struct CarMotionData
{
    pub f32 m_worldPositionX;               // World space X position
    pub f32 m_worldPositionY;               // World space Y position
    pub f32 m_worldPositionZ;               // World space Z position
    pub f32 m_worldVelocityX;               // Velocity in world space X
    pub f32 m_worldVelocityY;               // Velocity in world space Y
    pub f32 m_worldVelocityZ;               // Velocity in world space Z
    pub i16 m_worldForwardDirX;             // World space forward X direction (normalised)
    pub i16 m_worldForwardDirY;             // World space forward Y direction (normalised)
    pub i16 m_worldForwardDirZ;             // World space forward Z direction (normalised)
    pub i16 m_worldRightDirX;               // World space right X direction (normalised)
    pub i16 m_worldRightDirY;               // World space right Y direction (normalised)
    pub i16 m_worldRightDirZ;               // World space right Z direction (normalised)
    pub f32 m_gForceLateral;                // Lateral G-Force component
    pub f32 m_gForceLongitudinal;           // Longitudinal G-Force component
    pub f32 m_gForceVertical;               // Vertical G-Force component
    pub f32 m_yaw;                          // Yaw angle in radians
    pub f32 m_pitch;                        // Pitch angle in radians
    pub f32 m_roll;                         // Roll angle in radians
};

pub struct PacketMotionData
{
    pub PacketHeader m_header;              // Header

    pub CarMotionData m_carMotionData[22];  // Data for all cars on track

    // Extra player car ONLY data
    pub f32 m_suspensionPosition[4];        // Note: All wheel arrays have the following order:
    pub f32 m_suspensionVelocity[4];        // RL, RR, FL, FR
    pub f32 m_suspensionAcceleration[4];    // RL, RR, FL, FR
    pub f32 m_wheelSpeed[4];                // Speed of each wheel
    pub f32 m_wheelSlip[4];                 // Slip ratio for each wheel
    pub f32 m_localVelocityX;               // Velocity in local space
    pub f32 m_localVelocityY;               // Velocity in local space
    pub f32 m_localVelocityZ;               // Velocity in local space
    pub f32 m_angularVelocityX;             // Angular velocity x-component
    pub f32 m_angularVelocityY;             // Angular velocity y-component
    pub f32 m_angularVelocityZ;             // Angular velocity z-component
    pub f32 m_angularAccelerationX;         // Angular velocity x-component
    pub f32 m_angularAccelerationY;         // Angular velocity y-component
    pub f32 m_angularAccelerationZ;         // Angular velocity z-component
    pub f32 m_frontWheelsAngle;             // Current front wheels angle in radians
};

/**
 * # Session Packet
 * The session packet includes details about the current session in progress.
 * Frequency: 2 per second
 * Size: 632 bytes
 * Version: 1
 */
pub struct MarshalZone
{
    pub f32 m_zoneStart;                // Fraction (0..1) of way through the lap the marshal zone starts
    pub m_zoneFlag m_zoneFlag;
};

#[repr(i8)]
pub enum m_zoneFlag {
    Invalid = -1,
    None = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
    Red = 4,
};

pub struct WeatherForecastSample
{
    pub m_sessionType m_sessionType;    
    pub u8  m_timeOffset;               // Time in minutes the forecast is for
    pub m_weather m_weather;
    pub i8  m_trackTemperature;         // Track temp. in degrees Celsius
    pub i8  m_trackTemperatureChange;
    pub i8  m_airTemperature;           // Air temp. in degrees celsius
    pub i8  m_airTemperatureChange;
    pub u8  m_rainPercentage;           // Rain percentage (0-100)
};

#[repr(u8)]
pub enum m_sessionType {
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
};

#[repr(u8)]
pub enum m_weather {
    Clear = 0,
    LightCloud = 1,
    Overcast = 2,
    RainLight = 3,
    RainHeavy = 4,
    RainStorm = 5
};

#[repr(i8)]
pub enum m_trackTemperatureChange {
    Up = 0,
    Down = 1,
    None = 2
};

#[repr(i8)]
pub enum m_airTemperatureChange {
    Up = 0,
    Down = 1,
    None = 2
};

pub struct PacketSessionData
{
    pub PacketHeader m_header;          // Header

    pub m_weather m_weather;
    pub i8  m_trackTemperature;         // Track temp. in degrees celsius
    pub i8  m_airTemperature;           // Air temp. in degrees celsius
    pub u8  m_totalLaps;                // Total number of laps in this race
    pub u16 m_trackLength;              // Track length in metres
    pub m_sessionType m_sessionType;
    pub i8  m_trackId;                  // -1 for unknown, see appendix
    pub m_formula m_formula;
    pub u16 m_sessionTimeLeft;          // Time left in session in seconds
    pub u16 m_sessionDuration;          // Session duration in seconds
    pub u8  m_pitSpeedLimit;            // Pit speed limit in kilometres per hour
    pub u8  m_gamePaused;               // Whether the game is paused – network game only
    pub u8  m_isSpectating;             // Whether the player is spectating
    pub u8  m_spectatorCarIndex;        // Index of the car being spectated
    pub u8  m_sliProNativeSupport;      // SLI Pro support, 0 = inactive, 1 = active
    pub u8  m_numMarshalZones;          // Number of marshal zones to follow
    pub MarshalZone m_marshalZones[21]; // List of marshal zones – max 21
    pub m_safetyCarStatus m_safetyCarStatus;
    pub u8  m_networkGame;              // 0 = offline, 1 = online
    pub u8  m_numWeatherForecastSamples;// Number of weather samples to follow
    pub WeatherForecastSample m_weatherForecastSamples[56]; // Array of weather forecast samples
    pub u8  m_forecastAccuracy;         // 0 = Perfect, 1 = Approximate
    pub u8  m_aiDifficulty;             // AI Difficulty rating – 0-110
    pub u32 m_seasonLinkIdentifier;     // Identifier for season - persists across saves
    pub u32 m_weekendLinkIdentifier;    // Identifier for weekend - persists across saves
    pub u32 m_sessionLinkIdentifier;    // Identifier for session - persists across saves
    pub u8  m_pitStopWindowIdealLap;    // Ideal lap to pit on for current strategy (player)
    pub u8  m_pitStopWindowLatestLap;   // Latest lap to pit on for current strategy (player)
    pub u8  m_pitStopRejoinPosition;    // Predicted position to rejoin at (player)
    pub u8  m_steeringAssist;           // 0 = off, 1 = on
    pub u8  m_brakingAssist;            // 0 = off, 1 = low, 2 = medium, 3 = high
    pub u8  m_gearboxAssist;            // 1 = manual, 2 = manual & suggested gear, 3 = auto
    pub u8  m_pitAssist;                // 0 = off, 1 = on
    pub u8  m_pitReleaseAssist;         // 0 = off, 1 = on
    pub u8  m_ERSAssist;                // 0 = off, 1 = on
    pub u8  m_DRSAssist;                // 0 = off, 1 = on
    pub u8  m_dynamicRacingLine;        // 0 = off, 1 = corners only, 2 = full
    pub u8  m_dynamicRacingLineType;    // 0 = 2D, 1 = 3D
    pub u8  m_gameMode;                 // Game mode id - see appendix
    pub u8  m_ruleSet;                  // Ruleset - see appendix
    pub u32 m_timeOfDay;                // Local time of day - minutes since midnight
    pub m_sessionLength m_sessionLength;
};

#[repr(u8)]
pub enum m_formula {
    Modern = 0,
    Classic = 1,
    Formula2 = 2,
    Generic = 3,
    Beta = 4,
    Supercars = 5,
    Esports = 6,
    Formula22021 = 7
};

#[repr(u8)]
pub enum m_safetyCarStatus {
    None = 0,
    Full = 1,
    Virtual = 2,
    FormationLap = 3,
};

#[repr(u8)]
pub enum m_sessionLength {
     None = 0,
     VeryShort = 2,
     Short = 3,
     Medium = 4
     MediumLong = 5,
     Long = 6,
     Full = 7
};

/**
 * # Lap Data Packet
 * The lap data packet gives details of all the cars in the session.
 * Frequency: Rate as specified in menus
 * Size: 972 bytes
 * Version: 1
 */
pub struct LapData
{
    pub u32 m_lastLapTimeInMS;              // Last lap time in milliseconds
    pub u32 m_currentLapTimeInMS;           // Current time around the lap in milliseconds
    pub u16 m_sector1TimeInMS;              // Sector 1 time in milliseconds
    pub u16 m_sector2TimeInMS;              // Sector 2 time in milliseconds
    pub f32 m_lapDistance;                  // Distance vehicle is around current lap in metres – could be negative if line hasn’t been crossed yet
    pub f32 m_totalDistance;                // Total distance travelled in session in metres – could be negative if line hasn’t been crossed yet
    pub f32 m_safetyCarDelta;               // Delta in seconds for safety car
    pub u8  m_carPosition;                  // Car race position
    pub u8  m_currentLapNum;                // Current lap number
    pub m_pitStatus m_pitStatus;
    pub u8  m_numPitStops;                  // Number of pit stops taken in this race
    pub u8  m_sector;                       // 0 = sector1, 1 = sector2, 2 = sector3
    pub u8  m_currentLapInvalid;            // Current lap invalid - 0 = valid, 1 = invalid
    pub u8  m_penalties;                    // Accumulated time penalties in seconds to be added
    pub u8  m_warnings;                     // Accumulated number of warnings issued
    pub u8  m_numUnservedDriveThroughPens;  // Num drive through pens left to serve
    pub u8  m_numUnservedStopGoPens;        // Num stop go pens left to serve
    pub u8  m_gridPosition;                 // Grid position the vehicle started the race in
    pub m_driverStatus m_driverStatus;
    pub m_resultStatus m_resultStatus;
    pub u8  m_pitLaneTimerActive;           // Pit lane timing, 0 = inactive, 1 = active
    pub u16 m_pitLaneTimeInLaneInMS;        // If active, the current time spent in the pit lane in ms
    pub u16 m_pitStopTimerInMS;             // Time of the actual pit stop in ms
    pub u8  m_pitStopShouldServePen;        // Whether the car should serve a penalty at this stop
};

#[repr(u8)]
pub enum m_pitStatus {
    None = 0,
    Pitting = 1,
    InPitArea = 2,
};

#[repr(u8)]
pub enum m_driverStatus {
    InGarage = 0,
    FlyingLap = 1,
    InLap = 2,
    OutLap = 3,
    OnTrack = 4,
};

#[repr(u8)]
pub enum m_resultStatus {
    Invalid = 0,
    Inactive = 1,
    Active = 2,
    Finished = 3,
    DidNotFinish = 4,
    Disqualified = 5,
    NotClassified = 6,
    Retired = 7,
}

pub struct PacketLapData
{
    pub PacketHeader m_header;              // Header

    pub LapData m_lapData[22];              // Lap data for allpub  cars on track

    pub u8 m_timeTrialPBCarIdx;             // Index of Personal Best car in time trial (255 if invalid)
    pub u8 m_timeTrialRivalCarIdx;          // Index of Rival car in time trial (255 if invalid)
};

/**
 * # Event Packet
 * This packet gives details of events that happen during the course of a session.
 * Frequency: When the event occurs
 * Size: 40 bytes
 * Version: 1
 */

// The event details packet is different for each type of event.
// Make sure only the correct type is interpreted.
union EventDataDetails
{
    struct
    {
        pub u8  vehicleIdx;                 // Vehicle index of car achieving fastest lap
        pub f32 lapTime;                    // Lap time is in seconds
    } FastestLap;

    struct
    {
        pub u8  vehicleIdx;                 // Vehicle index of car retiring
    } Retirement;

    struct
    {
        pub u8  vehicleIdx;                 // Vehicle index of team mate
    } TeamMateInPits;

    struct
    {
        pub u8  vehicleIdx;                 // Vehicle index of the race winner
    } RaceWinner;

    struct
    {
        pub u8  penaltyType;                // Penalty type – see Appendices
        pub u8  infringementType;           // Infringement type – see Appendices
        pub u8  vehicleIdx;                 // Vehicle index of the car the penalty is applied to
        pub u8  otherVehicleIdx;            // Vehicle index of the other car involved
        pub u8  time;                       // Time gained, or time spent doing action in seconds
        pub u8  lapNum;                     // Lap the penalty occurred on
        pub u8  placesGained;               // Number of places gained by this
    } Penalty;

    struct
    {
        pub u8  vehicleIdx;                 // Vehicle index of the vehicle triggering speed trap
        pub f32 speed;                      // Top speed achieved in kilometres per hour
        pub u8  isOverallFastestInSession;  // Overall fastest speed in session = 1, otherwise 0
        pub u8  isDriverFastestInSession;   // Fastest speed for driver in session = 1, otherwise 0
        pub u8  fastestVehicleIdxInSession; // Vehicle index of the vehicle that is the fastest in this session
        pub f32 fastestSpeedInSession;      // Speed of the vehicle that is the fastest in this session
    } SpeedTrap;

    struct
    {
        pub u8  numLights;                  // Number of lights showing
    } StartLights;

    struct
    {
        pub u8  vehicleIdx;                 // Vehicle index of the vehicle serving drive through
    } DriveThroughPenaltyServed;

    struct
    {
        pub u8  vehicleIdx;                 // Vehicle index of the vehicle serving stop go
    } StopGoPenaltyServed;

    struct
    {
        pub u32 flashbackFrameIdentifier;   // Frame identifier flashed back to
        pub f32 flashbackSessionTime;       // Session time flashed back to
    } Flashback;

    struct
    {
        pub u32 m_buttonStatus;             // Bit flags specifying which buttons are being pressed currently - see appendices
    } Buttons;
};

pub struct PacketEventData
{
    pub PacketHeader m_header;              // Header

    pub String m_eventStringCode[4];        // Event string code, see below
    pub EventDataDetails m_eventDetails;    // Event details - should be interpreted differently for each type
};

/**
 * # Event String Codes
 */
pub enum m_eventStringCode: char[4]
{
    SessionStarted = "SSTA",                // Sent when the session starts
    SessionEnded = "SEND",                  // Sent when the session ends
    FastestLap = "FTLP",                    // When a driver achieves the fastest lap
    Retirement = "RTMT",                    // When a driver retires
    DRSenabled = "DRSE",                    // Race control have enabled DRS
    DRSdisabled = "DRSD",                   // Race control have disabled DRS
    TeamMateInPits = "TMPT",                // Your team mate has entered the pits
    ChequeredFlag = "CHQF",                 // The chequered flag has been waved
    RaceWinner = "RCWN",                    // The race winner is announced
    Penalty = "PENA",                       // A penalty has been issued – details in event
    SpeedTrap = "SPTP",                     // Speed trap has been triggered by fastest speed
    StartLights = "STLG",                   // Start lights – number shown
    LightsOut = "LGOT",                     // Lights out
    DriveThroughPenaltyServed = "DTSV",     // Drive through penalty served
    StopGoPenaltyServed = "SGSV",           // Stop go penalty served
    Flashback = "FLBK",                     // Flashback activated
    Buttons = "BUTN",                       // Button status changed
};

/**
 * # Participants Packet
 * This is a list of participants in the race. If the vehicle is controlled by AI, then the name will be the driver name. If this is a multiplayer game, the names will be the Steam Id on PC, or the LAN name if appropriate.
 * N.B. on Xbox One, the names will always be the driver name, on PS4 the name will be the LAN name if playing a LAN game, otherwise it will be the driver name. 
 * The array should be indexed by vehicle index.
 * Frequency: Every 5 seconds
 * Size: 1257 bytes
 * Version: 1
 */
pub struct ParticipantData
{
    pub u8  m_aiControlled;     // Whether the vehicle is AI (1) or Human (0) controlled
    pub u8  m_driverId;         // Driver id - see appendix, 255 if network human
    pub u8  m_networkId;        // Network id – unique identifier for network players
    pub u8  m_teamId;           // Team id - see appendix
    pub u8  m_myTeam;           // My team flag – 1 = My Team, 0 = otherwise
    pub u8  m_raceNumber;       // Race number of the car
    pub u8  m_nationality;      // Nationality of the driver
    pub String m_name[48];      // Name of participant in UTF-8 format – null terminated Will be truncated with … (U+2026) if too long
    pub u8  m_yourTelemetry;    // The player's UDP setting, 0 = restricted, 1 = public
};

pub struct PacketParticipantsData
{
    pub PacketHeader m_header;  // Header

    pub u8  m_numActiveCars;    // Number of active cars in the data – should match number of cars on HUD
    pub ParticipantData m_participants[22];
};

/**
 * # Car Setups Packet
 * This packet details the car setups for each vehicle in the session. Note that in multiplayer games, other player cars will appear as blank, you will only be able to see your car setup and AI cars.
 * Frequency: 2 per second
 * Size: 1102 bytes
 * Version: 1
 */
pub struct CarSetupData
{
    pub u8  m_frontWing;                // Front wing aero
    pub u8  m_rearWing;                 // Rear wing aero
    pub u8  m_onThrottle;               // Differential adjustment on throttle (percentage)
    pub u8  m_offThrottle;              // Differential adjustment off throttle (percentage)
    pub f32 m_frontCamber;              // Front camber angle (suspension geometry)
    pub f32 m_rearCamber;               // Rear camber angle (suspension geometry)
    pub f32 m_frontToe;                 // Front toe angle (suspension geometry)
    pub f32 m_rearToe;                  // Rear toe angle (suspension geometry)
    pub u8  m_frontSuspension;          // Front suspension
    pub u8  m_rearSuspension;           // Rear suspension
    pub u8  m_frontAntiRollBar;         // Front anti-roll bar
    pub u8  m_rearAntiRollBar;          // Front anti-roll bar
    pub u8  m_frontSuspensionHeight;    // Front ride height
    pub u8  m_rearSuspensionHeight;     // Rear ride height
    pub u8  m_brakePressure;            // Brake pressure (percentage)
    pub u8  m_brakeBias;                // Brake bias (percentage)
    pub f32 m_rearLeftTyrePressure;     // Rear left tyre pressure (PSI)
    pub f32 m_rearRightTyrePressure;    // Rear right tyre pressure (PSI)
    pub f32 m_frontLeftTyrePressure;    // Front left tyre pressure (PSI)
    pub f32 m_frontRightTyrePressure;   // Front right tyre pressure (PSI)
    pub u8  m_ballast;                  // Ballast
    pub f32 m_fuelLoad;                 // Fuel load
};

pub struct PacketCarSetupData
{
    pub PacketHeader m_header;          // Header

    pub CarSetupData m_carSetups[22];
};

/**
 * # Car Telemetry Packet
 * This packet details telemetry for all the cars in the race. It details various values that would be recorded on the car such as speed, throttle application, DRS etc. Note that the rev light configurations are presented separately as well and will mimic real life driver preferences.
 * Frequency: Rate as specified in menus
 * Size: 1347 bytes
 * Version: 1
 */
pub struct CarTelemetryData
{
    pub u16 m_speed;                            // Speed of car in kilometres per hour
    pub f32 m_throttle;                         // Amount of throttle applied (0.0 to 1.0)
    pub f32 m_steer;                            // Steering (-1.0 (full lock left) to 1.0 (full lock right))
    pub f32 m_brake;                            // Amount of brake applied (0.0 to 1.0)
    pub u8  m_clutch;                           // Amount of clutch applied (0 to 100)
    pub i8  m_gear;                             // Gear selected (1-8, N=0, R=-1)
    pub u16 m_engineRPM;                        // Engine RPM
    pub u8  m_drs;                              // 0 = off, 1 = on
    pub u8  m_revLightsPercent;                 // Rev lights indicator (percentage)
    pub u16 m_revLightsBitValue;                // Rev lights (bit 0 = leftmost LED, bit 14 = rightmost LED)
    pub u16 m_brakesTemperature[4];             // Brakes temperature (celsius)
    pub u8  m_tyresSurfaceTemperature[4];       // Tyres surface temperature (celsius)
    pub u8  m_tyresInnerTemperature[4];         // Tyres inner temperature (celsius)
    pub u16 m_engineTemperature;                // Engine temperature (celsius)
    pub f32 m_tyresPressure[4];                 // Tyres pressure (PSI)
    pub u8  m_surfaceType[4];                   // Driving surface, see appendices
};

pub struct PacketCarTelemetryData
{
    pub PacketHeader        m_header;           // Header

    pub CarTelemetryData    m_carTelemetryData[22];

    pub u8  m_mfdPanelIndex;                    // Index of MFD panel open - 255 = MFD closed Single player, race – 0 = Car setup, 1 = Pits 2 = Damage, 3 =  Engine, 4 = Temperatures May vary depending on game mode
    pub u8  m_mfdPanelIndexSecondaryPlayer;     // See above
    pub i8  m_suggestedGear;                    // Suggested gear for the player (1-8) 0 if no gear suggested
};

/**
 * # Car Status Packet
 * This packet details car statuses for all the cars in the race.
 * Frequency: Rate as specified in menus
 * Size: 1058 bytes
 * Version: 1
 */
pub struct CarStatusData
{
    pub m_tractionControl m_tractionControl;
    pub u8  m_antiLockBrakes;           // 0 (off) - 1 (on)
    pub m_fuelMix m_fuelMix;
    pub u8  m_frontBrakeBias;           // Front brake bias (percentage)
    pub u8  m_pitLimiterStatus;         // Pit limiter status - 0 = off, 1 = on
    pub f32 m_fuelInTank;               // Current fuel mass
    pub f32 m_fuelCapacity;             // Fuel capacity
    pub f32 m_fuelRemainingLaps;        // Fuel remaining in terms of laps (value on MFD)
    pub u16 m_maxRPM;                   // Cars max RPM, point of rev limiter
    pub u16 m_idleRPM;                  // Cars idle RPM
    pub u8  m_maxGears;                 // Maximum number of gears
    pub u8  m_drsAllowed;               // 0 = not allowed, 1 = allowed
    pub u16 m_drsActivationDistance;    // 0 = DRS not available, non-zero - DRS will be available in [X] metres
    pub m_actualTyreCompound m_actualTyreCompound;
    pub u8  m_visualTyreCompound;       // F1 visual (can be different from actual compound) 16 = soft, 17 = medium, 18 = hard, 7 = inter, 8 = wet F1 Classic – same as above F2 ‘19, 15 = wet, 19 – super soft, 20 = soft 21 = medium , 22 = hard
    pub u8  m_tyresAgeLaps;             // Age in laps of the current set of tyres
    pub m_zoneFlag m_vehicleFiaFlags;
    pub f32 m_ersStoreEnergy;           // ERS energy store in Joules
    pub m_ersDeployMode m_ersDeployMode;
    pub f32 m_ersHarvestedThisLapMGUK;  // ERS energy harvested this lap by MGU-K
    pub f32 m_ersHarvestedThisLapMGUH;  // ERS energy harvested this lap by MGU-H
    pub f32 m_ersDeployedThisLap;       // ERS energy deployed this lap
    pub u8  m_networkPaused;            // Whether the car is paused in a network game
};

#[repr(u8)]
pub enum m_actualTyreCompound
{
    Inter = 7,
    Wet = 8,
    Dry = 9,
    Wet = 10,
    SuperSoft = 11,
    Soft = 12,
    Medium = 13,
    Hard = 14,
    Wet = 15,
    C5 = 16,
    C4 = 17,
    C3 = 18,
    C2 = 19,
    C1 = 20,
};

// m_visualTyreCompound is a todo.

#[repr(u8)]
pub enum m_ersDeployMode {
    None = 0,
    Medium = 1,
    Hotlap = 2,
    Overtake = 3,
};

#[repr(u8)]
pub enum m_tractionControl {
    Off = 0,
    Medium = 1,
    Full = 2,
};

#[repr(u8)]
pub enum m_fuelMix {
    Lean = 0,
    Standard = 1,
    Rich = 2,
    Max = 3,
};

pub struct PacketCarStatusData
{
    pub PacketHeader m_header;          // Header

    pub CarStatusData m_carStatusData[22];
};

/**
 * # Final Classification Packet
 * This packet details the final classification at the end of the race, and the data will match with the post race results screen. This is especially useful for multiplayer games where it is not always possible to send lap times on the final frame because of network delay.
 * Frequency: Once at the end of a race
 * Size: 1015 bytes
 * Version: 1
 */
pub struct FinalClassificationData
{
    pub u8  m_position;             // Finishing position
    pub u8  m_numLaps;              // Number of laps completed
    pub u8  m_gridPosition;         // Grid position of the car
    pub u8  m_points;               // Number of points scored
    pub u8  m_numPitStops;          // Number of pit stops made
    pub m_resultStatus m_resultStatus;
    pub u32 m_bestLapTimeInMS;      // Best lap time of the session in milliseconds
    pub f64 m_totalRaceTime;        // Total race time in seconds without penalties
    pub u8  m_penaltiesTime;        // Total penalties accumulated in seconds
    pub u8  m_numPenalties;         // Number of penalties applied to this driver
    pub u8  m_numTyreStints;        // Number of tyres stints up to maximum
    pub u8  m_tyreStintsActual[8];  // Actual tyres used by this driver
    pub u8  m_tyreStintsVisual[8];  // Visual tyres used by this driver
    pub u8  m_tyreStintsEndLaps[8]; // The lap number stints end on
};

pub struct PacketFinalClassificationData
{
    pub PacketHeader m_header;      // Header

    pub u8  m_numCars;              // Number of cars in the final classification
    pub FinalClassificationData m_classificationData[22];
};

/**
 * # Lobby Info Packet
 * This packet details the players currently in a multiplayer lobby. It details each player’s selected car, any AI involved in the game and also the ready status of each of the participants.
 * Frequency: Two every second when in the lobby
 * Size: 1191 bytes
 * Version: 1
 */
pub struct LobbyInfoData
{
    pub u8  m_aiControlled;     // Whether the vehicle is AI (1) or Human (0) controlled
    pub u8  m_teamId;           // Team id - see appendix (255 if no team currently selected)
    pub u8  m_nationality;      // Nationality of the driver
    pub String m_name[48];      // Name of participant in UTF-8 format – null terminated Will be truncated with ... (U+2026) if too long
    pub u8  m_carNumber;        // Car number of the player
    pub m_readyStatus m_readyStatus;
};

#[repr(u8)]
pub enum m_readyStatus {
    NotReady = 0,
    Ready = 1,
    Spectating = 2,
};

pub struct PacketLobbyInfoData
{
    pub PacketHeader m_header;  // Header

    // Packet specific data
    pub u8  m_numPlayers;       // Number of players in the lobby data
    pub LobbyInfoData m_lobbyPlayers[22];
};


/**
 * # Car Damage Packet
 * This packet details car damage parameters for all the cars in the race.
 * Frequency: 2 per second
 * Size: 948 bytes
 * Version: 1
 */
pub struct CarDamageData
{
    pub f32 m_tyresWear[4];         // Tyre wear (percentage)
    pub u8  m_tyresDamage[4];       // Tyre damage (percentage)
    pub u8  m_brakesDamage[4];      // Brakes damage (percentage)
    pub u8  m_frontLeftWingDamage;  // Front left wing damage (percentage)
    pub u8  m_frontRightWingDamage; // Front right wing damage (percentage)
    pub u8  m_rearWingDamage;       // Rear wing damage (percentage)
    pub u8  m_floorDamage;          // Floor damage (percentage)
    pub u8  m_diffuserDamage;       // Diffuser damage (percentage)
    pub u8  m_sidepodDamage;        // Sidepod damage (percentage)
    pub u8  m_drsFault;             // Indicator for DRS fault, 0 = OK, 1 = fault
    pub u8  m_ersFault;             // Indicator for ERS fault, 0 = OK, 1 = fault
    pub u8  m_gearBoxDamage;        // Gear box damage (percentage)
    pub u8  m_engineDamage;         // Engine damage (percentage)
    pub u8  m_engineMGUHWear;       // Engine wear MGU-H (percentage)
    pub u8  m_engineESWear;         // Engine wear ES (percentage)
    pub u8  m_engineCEWear;         // Engine wear CE (percentage)
    pub u8  m_engineICEWear;        // Engine wear ICE (percentage)
    pub u8  m_engineMGUKWear;       // Engine wear MGU-K (percentage)
    pub u8  m_engineTCWear;         // Engine wear TC (percentage)
    pub u8  m_engineBlown;          // Engine blown, 0 = OK, 1 = fault
    pub u8  m_engineSeized;         // Engine seized, 0 = OK, 1 = fault
}

pub struct PacketCarDamageData
{
    pub PacketHeader m_header;      // Header

    pub CarDamageData m_carDamageData[22];
};

/**
 * # Session History Packet
 * This packet contains lap times and tyre usage for the session. **This packet works slightly differently to other packets. To reduce CPU and bandwidth, each packet relates to a specific vehicle and is sent every 1/20 s, and the vehicle being sent is cycled through. Therefore in a 20 car race you should receive an update for each vehicle at least once per second.**
 * Note that at the end of the race, after the final classification packet has been sent, a final bulk update of all the session histories for the vehicles in that session will be sent.
 * Frequency: 20 per second but cycling through cars
 * Size: 1155 bytes
 * Version: 1
 */
pub struct LapHistoryData
{
    pub u32 m_lapTimeInMS;          // Lap time in milliseconds
    pub u16 m_sector1TimeInMS;      // Sector 1 time in milliseconds
    pub u16 m_sector2TimeInMS;      // Sector 2 time in milliseconds
    pub u16 m_sector3TimeInMS;      // Sector 3 time in milliseconds
    pub u8  m_lapValidBitFlags;     // 0x01 bit set-lap valid, 0x02 bit set-sector 1 valid 0x04 bit set-sector 2 valid, 0x08 bit set-sector 3 valid
};

pub struct TyreStintHistoryData
{
    pub u8  m_endLap;               // Lap the tyre usage ends on (255 of current tyre)
    pub u8  m_tyreActualCompound;   // Actual tyres used by this driver
    pub u8  m_tyreVisualCompound;   // Visual tyres used by this driver
};

pub struct PacketSessionHistoryData
{
    pub PacketHeader m_header;      // Header

    pub u8  m_carIdx;               // Index of the car this lap data relates to
    pub u8  m_numLaps;              // Num laps in the data (including current partial lap)
    pub u8  m_numTyreStints;        // Number of tyre stints in the data

    pub u8  m_bestLapTimeLapNum;    // Lap the best lap time was achieved on
    pub u8  m_bestSector1LapNum;    // Lap the best Sector 1 time was achieved on
    pub u8  m_bestSector2LapNum;    // Lap the best Sector 2 time was achieved on
    pub u8  m_bestSector3LapNum;    // Lap the best Sector 3 time was achieved on

    pub LapHistoryData m_lapHistoryData[100]; // 100 laps of data max
    pub TyreStintHistoryData m_tyreStintsHistoryData[8];
};

/**
 * # Restricted data (Your Telemetry setting)
 * There is some data in the UDP that you may not want other players seeing if you are in a multiplayer game. This is controlled by the “Your Telemetry” setting in the Telemetry options. The options are:
 *     * Restricted (Default) – other players viewing the UDP data will not see values for your car
 *     * Public – all other players can see all the data for your car
 *     * Show online ID – this additional option allows other players to view your online ID / gamertag in their UDP output.
 * `Note: You can always see the data for the car you are driving regardless of the setting.`
 * The following data items are set to zero if the player driving the car in question has their “Your Telemetry” set to “Restricted”:
 * 
 * # Car status packet
 *     * m_fuelInTank
 *     * m_fuelCapacity
 *     * m_fuelMix
 *     * m_fuelRemainingLaps
 *     * m_frontBrakeBias
 *     * m_ersDeployMode
 *     * m_ersStoreEnergy
 *     * m_ersDeployedThisLap
 *     * m_ersHarvestedThisLapMGUK
 *     * m_ersHarvestedThisLapMGUH
 * 
 * # Car damage packet
 *     * m_frontLeftWingDamage
 *     * m_frontRightWingDamage
 *     * m_rearWingDamage
 *     * m_floorDamage
 *     * m_diffuserDamage
 *     * m_sidepodDamage
 *     * m_engineDamage
 *     * m_gearBoxDamage
 *     * m_tyresWear (All four wheels)
 *     * m_tyresDamage (All four wheels)
 *     * m_brakesDamage (All four wheels)
 *     * m_drsFault
 *     * m_engineMGUHWear
 *     * m_engineESWear
 *     * m_engineCEWear
 *     * m_engineICEWear
 *     * m_engineMGUKWear
 *     * m_engineTCWear
