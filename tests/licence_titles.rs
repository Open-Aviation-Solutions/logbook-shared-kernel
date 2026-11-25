// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::BTreeSet;

use icao_shared_kernel::{
    IssuedTitle, IssuingState, Licence, LicenceTitle, MinimumAges, PrivateOrCommercialCategory,
};
use logbook_shared_kernel::{
    AircraftCharacteristic, AircraftDescription, AuthorityId, AuthoritySpecificInformation,
    AuthorityTotal, Classification, CurrencyResult, DateRange, HeldLicenceTitle, LicenceTitleId,
    LicenceTitleProblem, LicenceTitleTerm, LicensingAuthority, Logbook, Problem, Questions, Record,
    Requirement, RequirementId, Term, TermId,
};
use time::macros::date;
use time::Date;

/// An authority with no State's rules: it issues the private pilot licence
/// for aeroplanes under its own name, and a licence of its own from age 16.
struct Example {
    id: AuthorityId,
}

impl LicensingAuthority for Example {
    fn id(&self) -> &AuthorityId {
        &self.id
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
        vec![
            LicenceTitleTerm::Annex1 {
                title: private(),
                term: Term {
                    id: TermId::new("private-aeroplane"),
                    name: "private pilot licence, aeroplane".into(),
                },
            },
            LicenceTitleTerm::National {
                term: Term {
                    id: TermId::new("own"),
                    name: "own licence".into(),
                },
                minimum_age: 16,
            },
            LicenceTitleTerm::National {
                term: Term {
                    id: TermId::new("older"),
                    name: "older licence".into(),
                },
                minimum_age: 17,
            },
        ]
    }

    fn ratings_and_endorsements(&self) -> Vec<Term> {
        vec![
            Term {
                id: TermId::new("second"),
                name: "second rating".into(),
            },
            Term {
                id: TermId::new("first"),
                name: "first endorsement".into(),
            },
        ]
    }

    fn minimum_ages(&self) -> MinimumAges {
        MinimumAges::default()
    }

    fn requirements(&self) -> Vec<Requirement> {
        Vec::new()
    }

    fn requirements_for(&self, _: &BTreeSet<TermId>) -> Vec<Requirement> {
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

fn private() -> LicenceTitle {
    LicenceTitle::Private(PrivateOrCommercialCategory::Aeroplane)
}

fn authority() -> Box<dyn LicensingAuthority> {
    Box::new(Example {
        id: AuthorityId::new("XX", "TEST").unwrap(),
    })
}

fn own() -> TermId {
    TermId::new("own")
}

fn held(title: TermId, on: Date) -> HeldLicenceTitle {
    HeldLicenceTitle::new(title, on)
}

fn licence(titles: Vec<IssuedTitle>) -> Licence {
    Licence::new(IssuingState::parse("XX").unwrap(), "TEST", "1", titles).unwrap()
}

fn problems(
    titles: Vec<IssuedTitle>,
    own: &[HeldLicenceTitle],
    born: Date,
) -> Vec<LicenceTitleProblem> {
    authority().licence_title_problems(&licence(titles), own, born)
}

#[test]
fn a_title_is_identified_by_annex_1s_title_or_the_authoritys_own_term() {
    let titles = authority().licence_titles();
    assert_eq!(titles[0].id(), LicenceTitleId::Annex1(private()));
    assert_eq!(titles[0].term().name, "private pilot licence, aeroplane");
    assert_eq!(titles[1].id(), LicenceTitleId::National(own()));
    assert_eq!(titles[1].term().id, own());
}

#[test]
fn an_own_title_is_held_from_its_date_of_first_issue() {
    let title = held(own(), date!(2020 - 03 - 10));
    assert!(!title.held_on(date!(2020 - 03 - 09)));
    assert!(title.held_on(date!(2020 - 03 - 10)));
    assert!(title.held_on(date!(2021 - 01 - 01)));
}

#[test]
fn titles_the_authority_issues_held_once_and_old_enough_break_no_rule() {
    let annex1 = IssuedTitle::new(private(), date!(2020 - 01 - 01));
    assert_eq!(
        problems(
            vec![annex1],
            &[held(own(), date!(2020 - 01 - 01))],
            date!(2000 - 01 - 01)
        ),
        []
    );
}

#[test]
fn an_annex_1_title_the_authority_leaves_out_is_not_issued() {
    let glider = IssuedTitle::new(LicenceTitle::Glider, date!(2020 - 01 - 01));
    assert_eq!(
        problems(vec![glider], &[], date!(2000 - 01 - 01)),
        [LicenceTitleProblem::NotIssued(LicenceTitleId::Annex1(
            LicenceTitle::Glider
        ))]
    );
}

#[test]
fn an_own_title_the_authority_does_not_list_is_not_issued() {
    let other = TermId::new("other");
    assert_eq!(
        problems(
            vec![],
            &[held(other.clone(), date!(2020 - 01 - 01))],
            date!(2000 - 01 - 01)
        ),
        [LicenceTitleProblem::NotIssued(LicenceTitleId::National(
            other
        ))]
    );
}

#[test]
fn an_own_title_listed_more_than_once_is_reported_once_and_each_listing_is_aged() {
    // Born 2004-06-15: 16 on 2020-06-15.
    assert_eq!(
        problems(
            vec![],
            &[
                held(own(), date!(2021 - 01 - 01)),
                held(own(), date!(2019 - 01 - 01)),
                held(own(), date!(2022 - 01 - 01)),
            ],
            date!(2004 - 06 - 15)
        ),
        [
            LicenceTitleProblem::Repeated(own()),
            LicenceTitleProblem::BeforeMinimumAge {
                title: own(),
                date_of_first_issue: date!(2019 - 01 - 01),
                minimum_age: 16,
            }
        ]
    );
}

#[test]
fn an_own_title_first_issued_before_the_minimum_age_is_reported() {
    // Born 2004-06-15: 16 on 2020-06-15.
    let born = date!(2004 - 06 - 15);
    assert_eq!(
        problems(vec![], &[held(own(), date!(2020 - 06 - 15))], born),
        []
    );
    assert_eq!(
        problems(vec![], &[held(own(), date!(2020 - 06 - 14))], born),
        [LicenceTitleProblem::BeforeMinimumAge {
            title: own(),
            date_of_first_issue: date!(2020 - 06 - 14),
            minimum_age: 16,
        }]
    );
}

#[test]
fn a_year_is_complete_on_1_march_for_one_born_on_29_february() {
    // Born 2004-02-29: 17 on 2021-03-01, not on 2021-02-28.
    let born = date!(2004 - 02 - 29);
    let older = TermId::new("older");
    let check = |on| problems(vec![], &[held(older.clone(), on)], born);
    assert_eq!(check(date!(2021 - 02 - 28)).len(), 1);
    assert_eq!(check(date!(2021 - 03 - 01)), []);
    // Before the birth the holder is not yet any age.
    assert_eq!(check(date!(2003 - 01 - 01)).len(), 1);
}
