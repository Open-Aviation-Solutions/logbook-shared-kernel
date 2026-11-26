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

/// An authority that warns of a title with no rating, and refuses a rating
/// without its endorsement.
struct Strict(AuthorityId);

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

impl LicensingAuthority for Strict {
    rest!();

    fn licence_problems(
        &self,
        titles: &[LicenceTitleId],
        ratings_and_endorsements: &[TermId],
    ) -> Vec<LicenceProblem> {
        let mut problems = Vec::new();
        if let Some(title) = titles.first() {
            if ratings_and_endorsements.is_empty() {
                problems.push(LicenceProblem::new(
                    LicenceProblemAt::Title(title.clone()),
                    ProblemKind::Warning,
                    "no rating",
                    "reg 1",
                ));
            }
        }
        if ratings_and_endorsements.contains(&TermId::new("rating"))
            && !ratings_and_endorsements.contains(&TermId::new("endorsement"))
        {
            problems.push(LicenceProblem::new(
                LicenceProblemAt::RatingOrEndorsement(TermId::new("rating")),
                ProblemKind::Refusal,
                "no endorsement",
                "reg 2",
            ));
        }
        problems
    }
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
fn an_authority_gives_a_warning_and_a_refusal_at_where_they_belong() {
    let a: Box<dyn LicensingAuthority> = Box::new(Strict(id()));
    let own = LicenceTitleId::National(TermId::new("own"));

    let warned = a.licence_problems(std::slice::from_ref(&own), &[]);
    assert_eq!(warned.len(), 1);
    assert_eq!(warned[0].at, LicenceProblemAt::Title(own));
    assert_eq!(warned[0].kind, ProblemKind::Warning);

    let refused = a.licence_problems(&[], &[TermId::new("rating")]);
    assert_eq!(refused.len(), 1);
    assert_eq!(
        refused[0].at,
        LicenceProblemAt::RatingOrEndorsement(TermId::new("rating"))
    );
    assert_eq!(refused[0].kind, ProblemKind::Refusal);
    assert_eq!(refused[0].regulation, "reg 2");
}
