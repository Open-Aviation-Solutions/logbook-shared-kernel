// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The titles of licence an authority issues, the titles a pilot holds under
//! a licence, and the rules over them that hold for any authority.
//!
//! Annex 1 establishes the pilot licences it names (icao-shared-kernel's
//! [`LicenceTitle`]); an authority may issue some of them, under its own
//! names, and some of its own that Annex 1 has none for. An Annex 1 title the
//! authority leaves out is one it does not issue.

use std::collections::BTreeSet;

use icao_shared_kernel::{age_on, Licence, LicenceTitle};
use time::Date;

use super::{Term, TermId};

/// One title of licence an authority issues: one of Annex 1's, or one of its
/// own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenceTitleTerm {
    /// One of Annex 1's, with the authority's own term for it. Its minimum
    /// age is the authority's [`minimum_ages`], checked by icao-shared-kernel's
    /// `Pilot`, not here.
    ///
    /// [`minimum_ages`]: super::LicensingAuthority::minimum_ages
    Annex1 { title: LicenceTitle, term: Term },
    /// One of the authority's own, which Annex 1 has no title for.
    National {
        term: Term,
        /// The age, in whole years, the holder must have reached when it is
        /// first issued.
        minimum_age: u8,
    },
}

impl LicenceTitleTerm {
    /// What identifies the title among the authority's.
    pub fn id(&self) -> LicenceTitleId {
        match self {
            Self::Annex1 { title, .. } => LicenceTitleId::Annex1(*title),
            Self::National { term, .. } => LicenceTitleId::National(term.id.clone()),
        }
    }

    /// The authority's term for the title, whichever kind it is.
    pub fn term(&self) -> &Term {
        match self {
            Self::Annex1 { term, .. } | Self::National { term, .. } => term,
        }
    }
}

/// Identifies a title of licence: Annex 1's by the title itself, an
/// authority's own by its term's id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LicenceTitleId {
    Annex1(LicenceTitle),
    National(TermId),
}

/// One of the authority's own titles held under a licence, from the date the
/// holder was first issued it: the date of first issue, not of the latest
/// document, so a reissued document does not move it.
///
/// Only the authority's own titles are held here. A held Annex 1 title is
/// icao-shared-kernel's `IssuedTitle` in the `Licence`'s titles, and "held on
/// a date" for it is `IssuedTitle::date_of_first_issue` on or before the date,
/// as [`held_on`](HeldLicenceTitle::held_on) is for an own title. An Annex 1
/// title is never recorded here: an own title whose id is an Annex 1 title's
/// term id is reported as not issued.
///
/// There is no expiry or suspension to record yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldLicenceTitle {
    pub title: TermId,
    pub date_of_first_issue: Date,
}

impl HeldLicenceTitle {
    pub fn new(title: TermId, date_of_first_issue: Date) -> Self {
        Self {
            title,
            date_of_first_issue,
        }
    }

    /// Whether the holder held it on `date`: it was first issued on or before.
    pub fn held_on(&self, date: Date) -> bool {
        self.date_of_first_issue <= date
    }
}

/// A rule the titles under a licence break, for any authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenceTitleProblem {
    /// The authority issues no such title: an Annex 1 title on the licence, or
    /// one of the own titles held.
    NotIssued(LicenceTitleId),
    /// An own title is listed more than once, so it carries more than one date
    /// of first issue.
    Repeated(TermId),
    /// The holder had not reached an own title's minimum age on its date of
    /// first issue.
    BeforeMinimumAge {
        title: TermId,
        date_of_first_issue: Date,
        minimum_age: u8,
    },
}

/// The problems with the titles under one licence, against the titles its
/// authority issues, for a holder born on `date_of_birth`: the Annex 1 titles
/// on `licence` the authority does not issue, then, for `own`, the authority's
/// own titles held, in the order they are listed, those it does not issue, one
/// listed more than once (reported once), and one first issued before its
/// minimum age (each listing is checked).
/// [`LicensingAuthority::licence_title_problems`] is this over the
/// authority's own titles.
///
/// Not the whole age check: an Annex 1 title's age is checked by
/// icao-shared-kernel's `Pilot`, built with the authority's
/// [`minimum_ages`](super::LicensingAuthority::minimum_ages), and
/// `Licence::new` refuses an Annex 1 title listed twice.
///
/// Age is counted as [`icao_shared_kernel::age_on`] counts it. The age is checked on the date of first issue, so it can miss an
/// applicant who was too young when they applied, though not one who was too
/// young when the title was issued.
///
/// [`LicensingAuthority::licence_title_problems`]: super::LicensingAuthority::licence_title_problems
pub fn licence_title_problems(
    issued: &[LicenceTitleTerm],
    licence: &Licence,
    own: &[HeldLicenceTitle],
    date_of_birth: Date,
) -> Vec<LicenceTitleProblem> {
    let mut problems = Vec::new();
    for held in licence.titles() {
        let issues = issued.iter().any(
            |term| matches!(term, LicenceTitleTerm::Annex1 { title, .. } if *title == held.title),
        );
        if !issues {
            problems.push(LicenceTitleProblem::NotIssued(LicenceTitleId::Annex1(
                held.title,
            )));
        }
    }
    let mut seen = BTreeSet::new();
    let mut repeated = BTreeSet::new();
    for held in own {
        let first = seen.insert(&held.title);
        if !first && repeated.insert(&held.title) {
            problems.push(LicenceTitleProblem::Repeated(held.title.clone()));
        }
        let term = issued.iter().find_map(|term| match term {
            LicenceTitleTerm::National { term, minimum_age } if term.id == held.title => {
                Some(*minimum_age)
            }
            _ => None,
        });
        match term {
            None if first => problems.push(LicenceTitleProblem::NotIssued(
                LicenceTitleId::National(held.title.clone()),
            )),
            Some(minimum_age)
                if age_on(date_of_birth, held.date_of_first_issue) < i32::from(minimum_age) =>
            {
                problems.push(LicenceTitleProblem::BeforeMinimumAge {
                    title: held.title.clone(),
                    date_of_first_issue: held.date_of_first_issue,
                    minimum_age,
                });
            }
            _ => {}
        }
    }
    problems
}
