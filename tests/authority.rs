// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod common;

use std::cell::Cell;
use std::collections::BTreeSet;

use common::{flight, session};
use icao_shared_kernel::{Capacity, MinimumAges};
use logbook_shared_kernel::{
    run, AircraftCharacteristic, AircraftDescription, AuthorityId, AuthoritySpecificInformation,
    AuthorityTotal, Classification, CurrencyResult, DateRange, Entry, EntryAccumulator,
    LicenceTitleTerm, LicensingAuthority, Logbook, Measure, MigrationRefused, Problem, Questions,
    Record, Requirement, RequirementId, Subject, Total,
};
use time::Date;

/// An authority with no State's rules, whose only rule is that a session
/// does not count. It counts how often it is asked.
struct FlightsOnly {
    id: AuthorityId,
    asked: Cell<usize>,
}

impl LicensingAuthority for FlightsOnly {
    fn id(&self) -> &AuthorityId {
        &self.id
    }

    fn counts(&self, entry: &Entry) -> bool {
        self.asked.set(self.asked.get() + 1);
        matches!(entry.subject(), Subject::Flight { .. })
    }

    fn classify(
        &self,
        _: &AircraftDescription,
        _: &BTreeSet<AircraftCharacteristic>,
    ) -> Option<Classification> {
        None
    }

    fn licence_titles(&self) -> Vec<LicenceTitleTerm> {
        Vec::new()
    }

    fn minimum_ages(&self) -> MinimumAges {
        MinimumAges::default()
    }

    fn requirements(&self) -> Vec<Requirement> {
        Vec::new()
    }

    fn currency(&self, _: Date, _: &Logbook) -> Vec<(RequirementId, CurrencyResult)> {
        Vec::new()
    }

    fn totals(&self, _: &Logbook, _: DateRange) -> Vec<AuthorityTotal> {
        Vec::new()
    }

    fn questions_for(&self, _: Record) -> Questions {
        Questions::new(1, Vec::new())
    }

    fn problems_for(&self, _: Record, _: &AuthoritySpecificInformation) -> Vec<Problem> {
        Vec::new()
    }
}

struct FlightTime;

impl Measure for FlightTime {
    fn measure(entry: &Entry) -> u64 {
        entry.flight_time().total_minutes().unsigned_abs()
    }
}

struct PicTime;

impl Measure for PicTime {
    fn measure(entry: &Entry) -> u64 {
        match entry.capacity() {
            Capacity::PilotInCommand => entry.flight_time().total_minutes().unsigned_abs(),
            _ => 0,
        }
    }
}

fn flights_only() -> FlightsOnly {
    FlightsOnly {
        id: AuthorityId::new("XX", "TEST").unwrap(),
        asked: Cell::new(0),
    }
}

fn authority() -> Box<dyn LicensingAuthority> {
    Box::new(flights_only())
}

#[test]
fn an_authority_is_held_as_a_trait_object() {
    assert_eq!(authority().id().to_string(), "XX-TEST");
}

#[test]
fn one_entry_reaches_every_accumulator_given_it() {
    let mut total = Total::<FlightTime>::new(DateRange::default());
    let mut pic = Total::<PicTime>::new(DateRange::default());
    run(
        authority().as_ref(),
        &[flight("e", Capacity::PilotInCommand)],
        &mut [
            &mut total as &mut dyn EntryAccumulator,
            &mut pic as &mut dyn EntryAccumulator,
        ],
    );
    assert_eq!((total.total(), pic.total()), (60, 60));
}

#[test]
fn what_the_authority_does_not_count_reaches_no_accumulator_and_only_that() {
    // Checked against two accumulators at once, and the gate is asked once
    // for each entry, not once for each accumulator.
    let authority = flights_only();
    let mut total = Total::<FlightTime>::new(DateRange::default());
    let mut pic = Total::<PicTime>::new(DateRange::default());
    run(
        &authority,
        &[
            session("s", Capacity::PilotInCommand),
            flight("f", Capacity::DualInstruction),
        ],
        &mut [
            &mut total as &mut dyn EntryAccumulator,
            &mut pic as &mut dyn EntryAccumulator,
        ],
    );
    assert_eq!((total.total(), pic.total()), (60, 0));
    assert_eq!(authority.asked.get(), 2);
}

fn migrate(
    value: serde_json::Value,
) -> Result<Option<AuthoritySpecificInformation>, MigrationRefused> {
    let entry = flight("e", Capacity::PilotInCommand);
    authority().migrate_information(
        Record::Entry(&entry),
        &AuthoritySpecificInformation::new(value),
    )
}

#[test]
fn the_default_has_nothing_to_migrate_at_the_current_version() {
    assert_eq!(migrate(serde_json::json!({"_version": 1})), Ok(None));
}

#[test]
fn the_default_refuses_a_newer_version_as_newer() {
    assert!(matches!(
        migrate(serde_json::json!({"_version": 2})),
        Err(MigrationRefused::Newer(_))
    ));
}

#[test]
fn the_default_refuses_an_older_version_as_too_old() {
    assert!(matches!(
        migrate(serde_json::json!({"_version": 0})),
        Err(MigrationRefused::TooOld(_))
    ));
}

#[test]
fn the_default_refuses_no_version_and_a_non_object_as_unreadable() {
    for value in [
        serde_json::json!({}),
        serde_json::json!("x"),
        serde_json::Value::Null,
    ] {
        assert!(matches!(
            migrate(value),
            Err(MigrationRefused::Unreadable(_))
        ));
    }
}
