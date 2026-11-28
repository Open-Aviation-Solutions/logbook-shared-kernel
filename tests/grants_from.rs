// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::BTreeSet;

use icao_shared_kernel::MinimumAges;
use logbook_shared_kernel::{
    AircraftCharacteristic, AircraftDescription, Assessment, AuthorityId,
    AuthoritySpecificInformation, AuthorityTotal, Classification, CurrencyResult, DateRange, Entry,
    Grant, LicenceTitleId, LicenceTitleTerm, LicensingAuthority, Logbook, Problem, Questions,
    Record, Requirement, RequirementId, Term, TermId,
};
use serde_json::json;
use time::Date;

mod common;
use common::{assessment, flight};
use icao_shared_kernel::Capacity;

/// An authority that leaves `grants_from` as it is.
struct Plain(AuthorityId);

macro_rules! rest {
    () => {
        fn id(&self) -> &AuthorityId {
            &self.0
        }
        fn counts(&self, _: &logbook_shared_kernel::Entry) -> bool {
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
            Questions::new(1, Vec::new()).unwrap()
        }
        fn problems_for(&self, _: Record, _: &AuthoritySpecificInformation) -> Vec<Problem> {
            Vec::new()
        }
    };
}

impl LicensingAuthority for Plain {
    rest!();
}

/// An authority that reads the purpose and outcome of a flight test from its
/// information, names the title for the aircraft's category when it has the
/// aircraft's classification, and grants a rating whether or not it has.
struct Granting(AuthorityId);

impl LicensingAuthority for Granting {
    rest!();

    fn grants_from(
        &self,
        assessment: &Assessment,
        _: &Entry,
        c: Option<&Classification>,
    ) -> Vec<Grant> {
        let Some(info) = assessment.authority_information.get(&self.0) else {
            return Vec::new();
        };
        let info = info.value();
        if info["kind"] != "flight test" || info["outcome"] != "pass" {
            return Vec::new();
        }
        let category = c.and_then(|c| c.counts_as.first()).map(|t| t.id.clone());
        category
            .map(|category| Grant::LicenceTitle(LicenceTitleId::National(category)))
            .into_iter()
            .chain([Grant::RatingOrEndorsement(TermId::new("night"))])
            .collect()
    }
}

fn id() -> AuthorityId {
    AuthorityId::new("XX", "TEST").unwrap()
}

fn classification() -> Classification {
    Classification {
        counts_as: vec![Term {
            id: TermId::new("aeroplane"),
            name: "Aeroplane".into(),
        }],
        ..Classification::default()
    }
}

fn assessed(kind: &str, outcome: &str) -> Vec<Grant> {
    assessed_as(kind, outcome, Some(&classification()))
}

fn assessed_as(kind: &str, outcome: &str, c: Option<&Classification>) -> Vec<Grant> {
    let entry = flight("e", Capacity::PilotInCommand);
    let mut a = assessment("a", &entry);
    a.authority_information.insert(
        id(),
        AuthoritySpecificInformation::new(json!({"kind": kind, "outcome": outcome})),
    );
    Granting(id()).grants_from(&a, &entry, c)
}

#[test]
fn an_authority_that_does_not_override_it_grants_nothing() {
    let entry = flight("e", Capacity::PilotInCommand);
    let a: Box<dyn LicensingAuthority> = Box::new(Plain(id()));
    assert!(a
        .grants_from(&assessment("a", &entry), &entry, Some(&classification()))
        .is_empty());
}

#[test]
fn a_passed_flight_test_grants_by_the_aircrafts_classification() {
    assert_eq!(
        assessed("flight test", "pass"),
        [
            Grant::LicenceTitle(LicenceTitleId::National(TermId::new("aeroplane"))),
            Grant::RatingOrEndorsement(TermId::new("night")),
        ]
    );
}

#[test]
fn a_failed_flight_test_or_a_flight_review_grants_nothing() {
    assert!(assessed("flight test", "fail").is_empty());
    assert!(assessed("flight review", "pass").is_empty());
}

#[test]
fn with_no_classification_a_passed_flight_test_grants_what_needs_none() {
    assert_eq!(
        assessed_as("flight test", "pass", None),
        [Grant::RatingOrEndorsement(TermId::new("night"))]
    );
}
