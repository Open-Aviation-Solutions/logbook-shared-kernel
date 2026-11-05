// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The logbook's common entry: the pilot's claim about a flight or a
//! session, in the terms ICAO's Annex 1 defines, with a section for each
//! authority whose rules the pilot logs it under.

mod approach;
mod authority_id;
mod day_night;
mod instrument_time;
mod logbook;
mod route;
mod section;

use std::collections::BTreeMap;
use std::fmt;

use icao_shared_kernel::{Aircraft, AircraftType, Capacity, FlightDuration, FlightRules};
use time::Date;
use uuid::Uuid;

pub use aircraft_type_descriptions::AircraftDescription;
pub use approach::{Approach, ApproachError};
pub use authority_id::AuthorityId;
pub use day_night::DayNight;
pub use icao_shared_kernel::{ApproachType, RnpApchMinima};
pub use instrument_time::InstrumentTime;
pub use logbook::{Assessment, AssessmentId, Logbook};
pub use route::Route;
pub use section::Section;

/// Identifies an entry. Opaque: a check only compares and reports it, and
/// the host maps it back to whatever it holds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryId(String);

impl EntryId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What an entry logs: a flight in an aircraft, or a session in a flight
/// simulation training device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    Flight {
        /// The kernel's `Flight`. One flight can have several entries, one
        /// for each capacity the pilot flew it in.
        flight: Uuid,
        /// Its type designator and registration marks.
        aircraft: Aircraft,
        /// The description of its type, as the host found it; `None` when
        /// it found none.
        description: Option<AircraftDescription>,
        /// Whether the pilot was "the sole occupant of an aircraft" (Annex 1,
        /// "solo flight time"). Required: a flight has an answer.
        sole_occupant: bool,
    },
    Session {
        /// The kernel's `FstdSession`.
        session: Uuid,
        /// The kernel's `FlightSimulationTrainingDevice`.
        device: Uuid,
        /// The type of aircraft the device represented, if recorded, and the
        /// description of that type as the host found it (`None` when it
        /// found none). A description is of a recorded type only.
        simulated: Option<(AircraftType, Option<AircraftDescription>)>,
    },
}

/// Everything an entry is made of, unchecked: build one and pass it to
/// [`Entry::new`]. Each field is described on [`Entry`]'s getter of the same
/// name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryParts {
    pub id: EntryId,
    pub date: Date,
    pub subject: Subject,
    pub route: Option<Route>,
    pub flight_rules: Option<FlightRules>,
    pub flight_time: FlightDuration,
    pub night: FlightDuration,
    pub capacity: Capacity,
    pub pilot_in_command: Option<String>,
    pub flight_crew: Vec<String>,
    pub instrument: InstrumentTime,
    pub approaches: Vec<Approach>,
    pub take_offs: DayNight<u32>,
    pub landings: DayNight<u32>,
    pub sections: BTreeMap<AuthorityId, Section>,
}

/// A logbook entry: the pilot's claim, in ICAO's terms, and each
/// authority's section of it. Built only through [`Entry::new`], so an
/// entry holds its rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    parts: EntryParts,
}

impl Entry {
    /// Check the parts and build the entry, or answer every rule they break.
    pub fn new(parts: EntryParts) -> Result<Self, Vec<EntryError>> {
        let mut errors = Vec::new();
        let blank = |name: &str| name.trim().is_empty();
        if parts.pilot_in_command.as_deref().is_some_and(blank) {
            errors.push(EntryError::EmptyPilotInCommand);
        }
        if parts.flight_crew.iter().any(|name| blank(name)) {
            errors.push(EntryError::EmptyFlightCrewName);
        }
        // Only a name counts as a person: an empty one is refused above.
        if let Subject::Flight {
            sole_occupant: true,
            ..
        } = parts.subject
        {
            if parts
                .pilot_in_command
                .as_deref()
                .is_some_and(|name| !blank(name))
            {
                errors.push(EntryError::SoleOccupantWithPilotInCommand);
            }
            if parts.flight_crew.iter().any(|name| !blank(name)) {
                errors.push(EntryError::SoleOccupantWithFlightCrew);
            }
            // These capacities need another pilot on board. Annex 1: a
            // co-pilot serves in a piloting capacity other than
            // pilot-in-command, and pilot-in-command under supervision is a
            // co-pilot acting under the pilot-in-command's supervision.
            // Dual instruction has the instructor on board (Annex 1; CASR
            // 1998 reg 61.010, dual flight: training "from a pilot
            // instructor occupying a flight control seat"). That a solo
            // entry is refused in these capacities is the owner's decision.
            if matches!(
                parts.capacity,
                Capacity::CoPilot
                    | Capacity::PilotInCommandUnderSupervision
                    | Capacity::DualInstruction
            ) {
                errors.push(EntryError::SoleOccupantInCapacityNeedingAnotherPilot(
                    parts.capacity,
                ));
            }
        }
        if parts.night > parts.flight_time {
            errors.push(EntryError::NightOverFlightTime);
        }
        let instrument = &parts.instrument;
        // `None`: more than a duration can hold, and so more than the flight
        // time.
        let instrument_time = match parts.subject {
            Subject::Flight { .. } => {
                if instrument.ground.total_minutes() > 0 {
                    errors.push(EntryError::GroundTimeOnFlight);
                }
                instrument.checked_flight()
            }
            Subject::Session { .. } => {
                if instrument.actual.total_minutes() > 0 || instrument.simulated.total_minutes() > 0
                {
                    errors.push(EntryError::InstrumentFlightTimeOnSession);
                }
                Some(instrument.ground)
            }
        };
        if instrument_time.is_none_or(|time| time > parts.flight_time) {
            errors.push(EntryError::InstrumentOverFlightTime);
        }
        if errors.is_empty() {
            Ok(Self { parts })
        } else {
            Err(errors)
        }
    }

    pub fn id(&self) -> &EntryId {
        &self.parts.id
    }

    /// "The date the flight began" (CASR 1998 reg 61.345(3)(a)), as the
    /// pilot's local date.
    pub fn date(&self) -> Date {
        self.parts.date
    }

    pub fn subject(&self) -> &Subject {
        &self.parts.subject
    }

    pub fn route(&self) -> Option<&Route> {
        self.parts.route.as_ref()
    }

    /// The flight rules it was flown under; `None`: not recorded.
    pub fn flight_rules(&self) -> Option<FlightRules> {
        self.parts.flight_rules
    }

    /// Annex 1's "flight time"; for a session, the simulated flight's.
    pub fn flight_time(&self) -> FlightDuration {
        self.parts.flight_time
    }

    /// The part of the flight time flown at night (Annex 1, "night"); the
    /// day part is the rest.
    pub fn night(&self) -> FlightDuration {
        self.parts.night
    }

    /// The capacity the pilot claims, in ICAO's terms as the logging
    /// authority defines them.
    pub fn capacity(&self) -> Capacity {
        self.parts.capacity
    }

    /// Who else was the pilot-in-command, by name (for dual instruction, or
    /// a co-pilot's flight); `None` when not recorded, on a session too.
    pub fn pilot_in_command(&self) -> Option<&str> {
        self.parts.pilot_in_command.as_deref()
    }

    /// The other members of the flight crew, by name.
    pub fn flight_crew(&self) -> &[String] {
        &self.parts.flight_crew
    }

    pub fn instrument(&self) -> &InstrumentTime {
        &self.parts.instrument
    }

    /// The instrument approaches the pilot conducted, as recorded.
    pub fn approaches(&self) -> &[Approach] {
        &self.parts.approaches
    }

    pub fn take_offs(&self) -> DayNight<u32> {
        self.parts.take_offs
    }

    pub fn landings(&self) -> DayNight<u32> {
        self.parts.landings
    }

    /// Each authority's section, keyed by the authority.
    pub fn sections(&self) -> &BTreeMap<AuthorityId, Section> {
        &self.parts.sections
    }
}

/// A rule an entry's parts break. Its `Display` is plain English, for logs.
/// Non-exhaustive: a rule added later is not a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntryError {
    /// A pilot-in-command is named, but the name is empty or only
    /// whitespace.
    EmptyPilotInCommand,
    /// A member of the flight crew is named, but the name is empty or only
    /// whitespace.
    EmptyFlightCrewName,
    /// The pilot was the sole occupant, but a pilot-in-command is named.
    SoleOccupantWithPilotInCommand,
    /// The pilot was the sole occupant, but flight crew are named.
    SoleOccupantWithFlightCrew,
    /// The pilot was the sole occupant, but the capacity (co-pilot,
    /// pilot-in-command under supervision, or dual instruction) needs
    /// another pilot on board.
    SoleOccupantInCapacityNeedingAnotherPilot(Capacity),
    /// More night than flight time.
    NightOverFlightTime,
    /// More instrument time than flight time: instrument flight time on a
    /// flight, instrument ground time on a session.
    InstrumentOverFlightTime,
    /// Instrument ground time on a flight, which is in an aircraft.
    GroundTimeOnFlight,
    /// Instrument flight time, actual or simulated, on a session, which is
    /// on the ground.
    InstrumentFlightTimeOnSession,
}

impl fmt::Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPilotInCommand => f.write_str("the pilot-in-command's name is empty"),
            Self::EmptyFlightCrewName => f.write_str("a flight crew member's name is empty"),
            Self::SoleOccupantWithPilotInCommand => {
                f.write_str("a sole occupant's entry names a pilot-in-command")
            }
            Self::SoleOccupantWithFlightCrew => {
                f.write_str("a sole occupant's entry names flight crew")
            }
            Self::SoleOccupantInCapacityNeedingAnotherPilot(capacity) => {
                let capacity = match capacity {
                    Capacity::CoPilot => "co-pilot",
                    Capacity::PilotInCommandUnderSupervision => {
                        "pilot-in-command under supervision"
                    }
                    Capacity::DualInstruction => "dual instruction",
                    _ => "a capacity that",
                };
                write!(
                    f,
                    "a sole occupant cannot log {capacity}: it needs another pilot on board"
                )
            }
            Self::NightOverFlightTime => f.write_str("the night time is more than the flight time"),
            Self::InstrumentOverFlightTime => {
                f.write_str("the instrument time is more than the flight time")
            }
            Self::GroundTimeOnFlight => f.write_str("a flight has instrument ground time"),
            Self::InstrumentFlightTimeOnSession => {
                f.write_str("a session has instrument flight time")
            }
        }
    }
}

impl std::error::Error for EntryError {}
