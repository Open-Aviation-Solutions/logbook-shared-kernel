// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Which of an entry's fields an authority asks, tested with CASA's answer
//! as logbook-shared-kernel#65 gives it (CASR 1998 reg 61.345(3)(g), (h) and
//! (4)(d)), as a fixture only: this crate holds no State's rules.

mod common;

use std::collections::BTreeSet;

use common::{flight, flight_parts, session};
use icao_shared_kernel::{Capacity, FlightDuration, MinimumAges};
use logbook_shared_kernel::{
    entry_field_applies, AircraftCharacteristic, AircraftDescription, Answer, AuthorityId,
    AuthoritySpecificInformation, AuthorityTotal, Classification, Condition, CurrencyResult,
    DateRange, Entry, EntryField, EntryParts, LicenceTitleTerm, LicensingAuthority, Logbook,
    Problem, Question, Questions, Record, Requirement, RequirementId, Term, TermId,
};
use time::Date;

macro_rules! rest {
    () => {
        fn id(&self) -> &AuthorityId {
            &self.0
        }
        fn counts(&self, _: &Entry) -> bool {
            true
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
        fn ratings_and_endorsements(&self) -> Vec<Term> {
            Vec::new()
        }
        fn minimum_ages(&self) -> MinimumAges {
            MinimumAges::default()
        }
        fn requirements_for(&self, _: &BTreeSet<TermId>) -> Vec<Requirement> {
            Vec::new()
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
    };
}

/// An authority that leaves `entry_fields` as it is.
struct Plain(AuthorityId);

impl LicensingAuthority for Plain {
    rest!();
}

/// CASA's answer, as logbook-shared-kernel#65 gives it.
struct Casa(AuthorityId);

impl LicensingAuthority for Casa {
    rest!();

    fn entry_fields(&self) -> Vec<(EntryField, Condition)> {
        vec![
            (
                EntryField::Instructor,
                Condition::Any(vec![
                    Condition::Holds(flight_instructor()),
                    Condition::All(vec![
                        Condition::Holds(simulator_instructor()),
                        Condition::Session,
                    ]),
                ]),
            ),
            (
                EntryField::Examiner,
                Condition::All(vec![Condition::Holds(flight_examiner()), Condition::Flight]),
            ),
        ]
    }
}

fn flight_instructor() -> TermId {
    TermId::new("flight-instructor")
}

fn simulator_instructor() -> TermId {
    TermId::new("simulator-instructor")
}

fn flight_examiner() -> TermId {
    TermId::new("flight-examiner")
}

fn authority(id: &str) -> AuthorityId {
    AuthorityId::new("XX", id).unwrap()
}

fn holding(ids: &[TermId]) -> BTreeSet<TermId> {
    ids.iter().cloned().collect()
}

/// Whether `field` is asked of `entry` under CASA's answer alone, the pilot
/// holding `held`.
fn casa_asks(field: EntryField, entry: &Entry, held: &[TermId]) -> bool {
    let casa = Casa(authority("CASA"));
    let held = holding(held);
    entry_field_applies(field, entry, [(&casa as &dyn LicensingAuthority, &held)])
}

#[test]
fn an_authority_that_does_not_override_it_asks_every_field() {
    let plain = Plain(authority("TEST"));
    let none = BTreeSet::new();
    let entry = flight("e", Capacity::PilotInCommand);
    for field in [EntryField::Instructor, EntryField::Examiner] {
        assert!(entry_field_applies(
            field,
            &entry,
            [(&plain as &dyn LicensingAuthority, &none)]
        ));
    }
}

#[test]
fn with_no_authority_every_field_is_asked() {
    let entry = flight("e", Capacity::PilotInCommand);
    assert!(entry_field_applies(EntryField::Instructor, &entry, []));
}

#[test]
fn instructor_time_is_asked_of_a_flight_instructor_on_a_flight_or_a_session() {
    let held = [flight_instructor()];
    let flight = flight("e", Capacity::PilotInCommand);
    let session = session("s", Capacity::PilotInCommand);
    assert!(casa_asks(EntryField::Instructor, &flight, &held));
    assert!(casa_asks(EntryField::Instructor, &session, &held));
    assert!(!casa_asks(EntryField::Instructor, &flight, &[]));
}

#[test]
fn instructor_time_is_asked_of_a_simulator_instructor_on_a_session_only() {
    let held = [simulator_instructor()];
    let session = session("s", Capacity::PilotInCommand);
    let flight = flight("e", Capacity::PilotInCommand);
    assert!(casa_asks(EntryField::Instructor, &session, &held));
    assert!(!casa_asks(EntryField::Instructor, &flight, &held));
}

#[test]
fn examiner_time_is_asked_of_a_flight_examiner_on_a_flight_only() {
    let held = [flight_examiner()];
    let flight = flight("e", Capacity::PilotInCommand);
    let session = session("s", Capacity::PilotInCommand);
    assert!(casa_asks(EntryField::Examiner, &flight, &held));
    assert!(!casa_asks(EntryField::Examiner, &session, &held));
    assert!(!casa_asks(
        EntryField::Examiner,
        &flight,
        &[flight_instructor()]
    ));
}

#[test]
fn a_field_is_asked_when_any_authority_asks_it() {
    let casa = Casa(authority("CASA"));
    let plain = Plain(authority("TEST"));
    let none = BTreeSet::new();
    let entry = flight("e", Capacity::PilotInCommand);
    assert!(!entry_field_applies(
        EntryField::Examiner,
        &entry,
        [(&casa as &dyn LicensingAuthority, &none)]
    ));
    assert!(entry_field_applies(
        EntryField::Examiner,
        &entry,
        [
            (&casa as &dyn LicensingAuthority, &none),
            (&plain as &dyn LicensingAuthority, &none),
        ]
    ));
}

#[test]
fn a_value_in_a_field_not_asked_is_kept() {
    let minutes = FlightDuration::new(30).unwrap();
    let entry = Entry::new(EntryParts {
        examiner: minutes,
        ..flight_parts("e", Capacity::PilotInCommand)
    })
    .unwrap();
    assert!(!casa_asks(EntryField::Examiner, &entry, &[]));
    assert_eq!(entry.examiner(), minutes);
}

#[test]
fn holds_never_holds_in_a_question() {
    let questions = Questions::new(
        1,
        vec![Question {
            key: "k",
            label: "A question for an instructor".into(),
            regulation: "reg 1",
            answer: Answer::YesNo,
            applies_when: Some(Condition::Holds(flight_instructor())),
            required: true,
        }],
    );
    let entry = session("s", Capacity::PilotInCommand);
    assert!(questions
        .applicable_for(Record::Entry(&entry), None)
        .is_empty());
}
