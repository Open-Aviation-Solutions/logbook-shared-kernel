// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::BTreeSet;

use icao_shared_kernel::MinimumAges;
use logbook_shared_kernel::{
    AircraftCharacteristic, AircraftDescription, AuthorityId, AuthoritySpecificInformation,
    AuthorityTotal, Classification, CurrencyResult, DateRange, LicenceProblem, LicenceProblemAt,
    LicenceTitleId, LicenceTitleTerm, LicensingAuthority, Logbook, Problem, ProblemKind, Questions,
    Record, Requirement, RequirementId, Term, TermId,
};
use time::Date;

/// An authority that leaves `licence_problems` as it is.
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
            Questions::new(1, Vec::new())
        }
        fn problems_for(&self, _: Record, _: &AuthoritySpecificInformation) -> Vec<Problem> {
            Vec::new()
        }
    };
}

impl LicensingAuthority for Plain {
    rest!();
}

fn id() -> AuthorityId {
    AuthorityId::new("XX", "TEST").unwrap()
}

#[test]
fn an_authority_that_does_not_override_it_has_no_problems() {
    let a: Box<dyn LicensingAuthority> = Box::new(Plain(id()));
    assert!(a
        .licence_problems(&[LicenceTitleId::National(TermId::new("own"))], &[])
        .is_empty());
}

#[test]
fn a_licence_problem_keeps_what_it_is_given() {
    let at = LicenceProblemAt::RatingOrEndorsement(TermId::new("rating"));
    let p = LicenceProblem::new(at.clone(), ProblemKind::Refusal, "no endorsement", "reg 2");
    assert_eq!(p.at, at);
    assert_eq!(p.kind, ProblemKind::Refusal);
    assert_eq!(p.text, "no endorsement");
    assert_eq!(p.regulation, "reg 2");
}
