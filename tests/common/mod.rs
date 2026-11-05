// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Entries for the tests, with no State's rules.

// Each test file is its own crate, and uses only some of these.
#![allow(dead_code)]

use std::collections::BTreeMap;

use icao_shared_kernel::{
    Aircraft, AircraftRegistration, AircraftType, Capacity, FlightDuration, NationalityMark,
};
use logbook_shared_kernel::{DayNight, Entry, EntryId, EntryParts, InstrumentTime, Subject};
use time::macros::date;
use uuid::Uuid;

/// A Cessna 172, VH-ABC.
pub fn c172() -> Aircraft {
    Aircraft::new(
        AircraftType::parse("C172").unwrap(),
        AircraftRegistration::new(NationalityMark::parse("VH").unwrap(), "ABC").unwrap(),
    )
}

/// An hour's flight, flown in `capacity`.
pub fn flight(id: &str, capacity: Capacity) -> Entry {
    Entry::new(flight_parts(id, capacity)).unwrap()
}

/// An hour's session, flown in `capacity`.
pub fn session(id: &str, capacity: Capacity) -> Entry {
    Entry::new(session_parts(id, capacity)).unwrap()
}

/// The parts of [`flight`], with someone else on board.
pub fn flight_parts(id: &str, capacity: Capacity) -> EntryParts {
    EntryParts {
        id: EntryId::new(id),
        date: date!(2026 - 11 - 05),
        subject: Subject::Flight {
            flight: Uuid::nil(),
            aircraft: c172(),
            description: None,
            sole_occupant: false,
        },
        route: None,
        flight_rules: None,
        flight_time: FlightDuration::new(60).unwrap(),
        night: FlightDuration::new(0).unwrap(),
        capacity,
        pilot_in_command: None,
        flight_crew: Vec::new(),
        instrument: InstrumentTime::default(),
        approaches: Vec::new(),
        take_offs: DayNight::default(),
        landings: DayNight::default(),
        sections: BTreeMap::new(),
    }
}

/// The parts of [`session`].
pub fn session_parts(id: &str, capacity: Capacity) -> EntryParts {
    EntryParts {
        subject: Subject::Session {
            session: Uuid::nil(),
            device: Uuid::nil(),
            simulated: None,
        },
        ..flight_parts(id, capacity)
    }
}
