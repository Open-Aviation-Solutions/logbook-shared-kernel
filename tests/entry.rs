// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod common;

use aircraft_type_descriptions::{AirframeKind, EngineType};
use common::{c172, flight_parts, session_parts};
use icao_shared_kernel::{AircraftType, Capacity, FlightDuration, LocationIndicator};
use logbook_shared_kernel::{
    AircraftDescription, Approach, ApproachType, Entry, EntryError, EntryParts, InstrumentTime,
    Subject,
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
            aircraft: c172(),
            description: None,
            sole_occupant,
        },
        ..flight_parts("e", Capacity::PilotInCommand)
    }
}

fn session() -> EntryParts {
    session_parts("e", Capacity::PilotInCommand)
}

fn description() -> AircraftDescription {
    AircraftDescription::new(AirframeKind::LandPlane, 1, EngineType::Piston)
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
fn a_sole_occupant_is_refused_in_a_capacity_needing_another_pilot() {
    for capacity in [
        Capacity::CoPilot,
        Capacity::PilotInCommandUnderSupervision,
        Capacity::DualInstruction,
    ] {
        let parts = EntryParts {
            capacity,
            ..flight(true)
        };
        assert_eq!(
            refused(parts),
            [EntryError::SoleOccupantInCapacityNeedingAnotherPilot(
                capacity
            )]
        );
    }
}

#[test]
fn a_capacity_needing_another_pilot_is_accepted_with_one_on_board() {
    for capacity in [
        Capacity::CoPilot,
        Capacity::PilotInCommandUnderSupervision,
        Capacity::DualInstruction,
    ] {
        let parts = EntryParts {
            capacity,
            ..flight(false)
        };
        assert!(Entry::new(parts).is_ok());
    }
}

#[test]
fn a_session_with_a_type_and_a_description_a_type_alone_or_neither_is_accepted() {
    let c172 = || AircraftType::parse("C172").unwrap();
    for simulated in [
        Some((c172(), Some(description()))),
        Some((c172(), None)),
        None,
    ] {
        let subject = Subject::Session {
            session: Uuid::nil(),
            device: Uuid::nil(),
            simulated,
        };
        assert!(Entry::new(EntryParts {
            subject,
            ..session()
        })
        .is_ok());
    }
}

#[test]
fn a_sole_occupant_as_co_pilot_is_refused_with_the_other_sole_occupant_refusals() {
    let parts = EntryParts {
        capacity: Capacity::CoPilot,
        pilot_in_command: Some("A. Instructor".into()),
        ..flight(true)
    };
    assert_eq!(
        refused(parts),
        [
            EntryError::SoleOccupantWithPilotInCommand,
            EntryError::SoleOccupantInCapacityNeedingAnotherPilot(Capacity::CoPilot)
        ]
    );
}

#[test]
fn an_empty_name_is_refused_and_is_not_counted_as_a_person() {
    for name in ["", "  "] {
        let parts = EntryParts {
            pilot_in_command: Some(name.into()),
            flight_crew: vec!["A. Crew".into(), name.into()],
            ..flight(false)
        };
        assert_eq!(
            refused(parts),
            [
                EntryError::EmptyPilotInCommand,
                EntryError::EmptyFlightCrewName
            ],
            "{name:?}"
        );
        let parts = EntryParts {
            pilot_in_command: Some(name.into()),
            flight_crew: vec![name.into()],
            ..flight(true)
        };
        assert_eq!(
            refused(parts),
            [
                EntryError::EmptyPilotInCommand,
                EntryError::EmptyFlightCrewName
            ],
            "{name:?}"
        );
    }
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
fn an_approach_holds_its_procedure_aerodrome_and_whether_it_circled() {
    let aerodrome = LocationIndicator::parse("YSBK").unwrap();
    let straight_in = Approach::new(ApproachType::Ils, aerodrome.clone(), false);
    let circled = Approach::new(ApproachType::Ils, aerodrome, true);
    assert_eq!(circled.procedure(), ApproachType::Ils);
    assert_eq!(*circled.aerodrome(), "YSBK");
    assert!(circled.circled());
    assert!(!straight_in.circled());
    assert_ne!(straight_in, circled);
}
