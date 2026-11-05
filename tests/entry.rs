// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod common;

use common::{flight_parts, session_parts};
use icao_shared_kernel::{Aircraft, Capacity, FlightDuration};
use logbook_shared_kernel::{
    Approach, ApproachError, ApproachType, Entry, EntryError, EntryParts, InstrumentTime, Subject,
};
use uuid::Uuid;

fn minutes(minutes: i64) -> FlightDuration {
    FlightDuration::new(minutes).unwrap()
}

/// An hour's flight as pilot-in-command, the pilot its sole occupant or not.
fn flight(sole_occupant: bool) -> EntryParts {
    EntryParts {
        subject: Subject::Flight {
            flight: Uuid::nil(),
            aircraft: Aircraft::create("C172", "VH", "ABC").unwrap(),
            description: None,
            sole_occupant,
        },
        ..flight_parts("e", Capacity::PilotInCommand)
    }
}

fn session() -> EntryParts {
    session_parts("e", Capacity::PilotInCommand)
}

fn refused(parts: EntryParts) -> Vec<EntryError> {
    Entry::new(parts).unwrap_err()
}

#[test]
fn a_sole_occupant_is_refused_with_a_pilot_in_command_named() {
    let parts = EntryParts {
        pilot_in_command: Some("A. Instructor".into()),
        ..flight(true)
    };
    assert_eq!(refused(parts), [EntryError::SoleOccupantWithPilotInCommand]);
}

#[test]
fn a_sole_occupant_is_refused_with_flight_crew_named() {
    let parts = EntryParts {
        flight_crew: vec!["A. Crew".into()],
        ..flight(true)
    };
    assert_eq!(refused(parts), [EntryError::SoleOccupantWithFlightCrew]);
}

#[test]
fn a_sole_occupant_with_no_one_else_named_is_accepted() {
    assert!(Entry::new(flight(true)).is_ok());
}

#[test]
fn others_may_be_named_when_the_pilot_was_not_the_sole_occupant() {
    let parts = EntryParts {
        pilot_in_command: Some("A. Instructor".into()),
        flight_crew: vec!["A. Crew".into()],
        ..flight(false)
    };
    assert!(Entry::new(parts).is_ok());
}

#[test]
fn night_over_the_flight_time_is_refused_and_night_equal_to_it_accepted() {
    let over = EntryParts {
        night: minutes(61),
        ..flight(false)
    };
    assert_eq!(refused(over), [EntryError::NightOverFlightTime]);
    let equal = EntryParts {
        night: minutes(60),
        ..flight(false)
    };
    assert!(Entry::new(equal).is_ok());
}

#[test]
fn instrument_flight_time_over_the_flight_time_is_refused_and_equal_to_it_accepted() {
    let instrument = |actual, simulated| EntryParts {
        instrument: InstrumentTime {
            actual: minutes(actual),
            simulated: minutes(simulated),
            ..InstrumentTime::default()
        },
        ..flight(false)
    };
    assert_eq!(
        refused(instrument(31, 30)),
        [EntryError::InstrumentOverFlightTime]
    );
    assert!(Entry::new(instrument(30, 30)).is_ok());
}

#[test]
fn instrument_flight_time_too_large_to_add_up_is_refused() {
    let parts = EntryParts {
        instrument: InstrumentTime {
            actual: minutes(i64::MAX),
            simulated: minutes(1),
            ..InstrumentTime::default()
        },
        ..flight(false)
    };
    assert_eq!(refused(parts), [EntryError::InstrumentOverFlightTime]);
}

#[test]
fn instrument_ground_time_on_a_flight_is_refused() {
    let parts = EntryParts {
        instrument: InstrumentTime {
            ground: minutes(1),
            ..InstrumentTime::default()
        },
        ..flight(false)
    };
    assert_eq!(refused(parts), [EntryError::GroundTimeOnFlight]);
}

#[test]
fn instrument_flight_time_on_a_session_is_refused() {
    for instrument in [
        InstrumentTime {
            actual: minutes(1),
            ..InstrumentTime::default()
        },
        InstrumentTime {
            simulated: minutes(1),
            ..InstrumentTime::default()
        },
    ] {
        let parts = EntryParts {
            instrument,
            ..session()
        };
        assert_eq!(refused(parts), [EntryError::InstrumentFlightTimeOnSession]);
    }
}

#[test]
fn instrument_ground_time_over_a_sessions_flight_time_is_refused_and_equal_to_it_accepted() {
    let ground = |ground| EntryParts {
        instrument: InstrumentTime {
            ground: minutes(ground),
            ..InstrumentTime::default()
        },
        ..session()
    };
    assert_eq!(refused(ground(61)), [EntryError::InstrumentOverFlightTime]);
    assert!(Entry::new(ground(60)).is_ok());
}

#[test]
fn every_rule_broken_is_reported() {
    let parts = EntryParts {
        pilot_in_command: Some("A. Instructor".into()),
        night: minutes(61),
        ..flight(true)
    };
    assert_eq!(
        refused(parts),
        [
            EntryError::SoleOccupantWithPilotInCommand,
            EntryError::NightOverFlightTime
        ]
    );
}

#[test]
fn an_approachs_aerodrome_is_a_4_letter_icao_location_indicator() {
    assert!(Approach::new(ApproachType::Ils, "YSBK").is_ok());
    for aerodrome in ["SYD", "YS", "YSBKX", "ysbk", "YS1K"] {
        assert_eq!(
            Approach::new(ApproachType::Ils, aerodrome),
            Err(ApproachError::Aerodrome(aerodrome.into())),
            "{aerodrome}"
        );
    }
}
