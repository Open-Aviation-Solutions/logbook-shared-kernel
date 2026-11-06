// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod common;

use std::cell::Cell;

use common::{flight, session};
use icao_shared_kernel::Capacity;
use logbook_shared_kernel::{
    run, AircraftDescription, AuthorityId, Classification, CurrencyResult, DateRange, Entry,
    EntryAccumulator, Figure, Form, LicensingAuthority, Logbook, Measure, Problem, Record,
    Requirement, RequirementId, Subject, Total,
};
use time::Date;

/// An authority with no State's rules, whose only rule is that a session
/// does not count. It counts how often it is asked.
struct FlightsOnly {
    authorities: Vec<AuthorityId>,
    asked: Cell<usize>,
}

impl LicensingAuthority for FlightsOnly {
    fn authorities(&self) -> &[AuthorityId] {
        &self.authorities
    }

    fn counts(&self, entry: &Entry) -> bool {
        self.asked.set(self.asked.get() + 1);
        matches!(entry.subject(), Subject::Flight { .. })
    }

    fn classify(&self, _: &AircraftDescription) -> Option<Classification> {
        None
    }

    fn requirements(&self) -> Vec<Requirement> {
        Vec::new()
    }

    fn currency(&self, _: Date, _: &Logbook) -> Vec<(RequirementId, CurrencyResult)> {
        Vec::new()
    }

    fn totals(&self, _: &Logbook, _: DateRange) -> Vec<Figure> {
        Vec::new()
    }

    fn section_form(&self, _: &AuthorityId, _: Record) -> Form {
        Form {
            version: 1,
            questions: Vec::new(),
        }
    }

    fn validate_section(&self, _: &AuthorityId, _: Record) -> Vec<Problem> {
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
        authorities: vec![AuthorityId::new("XX", "TEST").unwrap()],
        asked: Cell::new(0),
    }
}

fn authority() -> Box<dyn LicensingAuthority> {
    Box::new(flights_only())
}

#[test]
fn an_authority_is_held_as_a_trait_object() {
    assert_eq!(authority().authorities()[0].to_string(), "XX-TEST");
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
