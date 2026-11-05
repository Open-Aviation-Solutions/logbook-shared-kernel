// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod common;

use common::{flight, session};
use icao_shared_kernel::Capacity;
use logbook_shared_kernel::{
    run, AircraftDescription, AuthorityId, Classification, CurrencyResult, DateRange, Entry,
    EntryAccumulator, Figure, Form, LicensingAuthority, Logbook, Measure, Problem, Requirement,
    RequirementId, Subject, Total,
};
use time::Date;

/// An authority with no State's rules, whose only rule is that a session
/// does not count.
struct FlightsOnly(Vec<AuthorityId>);

impl LicensingAuthority for FlightsOnly {
    fn authorities(&self) -> &[AuthorityId] {
        &self.0
    }

    fn counts(&self, entry: &Entry) -> bool {
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

    fn section_form(&self, _: &Entry) -> Form {
        Form {
            questions: Vec::new(),
        }
    }

    fn validate_section(&self, _: &Entry) -> Vec<Problem> {
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

fn authority() -> Box<dyn LicensingAuthority> {
    Box::new(FlightsOnly(vec![AuthorityId::new("XX", "TEST").unwrap()]))
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
    // Checked against two accumulators at once: the gate runs once, outside
    // the loop over the accumulators.
    let mut total = Total::<FlightTime>::new(DateRange::default());
    let mut pic = Total::<PicTime>::new(DateRange::default());
    run(
        authority().as_ref(),
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
}
