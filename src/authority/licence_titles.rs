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

use icao_shared_kernel::LicenceTitle;
use time::Date;

use super::{Term, TermId};

/// One title of licence an authority issues: one of Annex 1's, or one of its
/// own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenceTitleTerm {
    /// One of Annex 1's, with the authority's own term for it. Its minimum
    /// age is checked by icao-shared-kernel's `Pilot`, with the State's ages
    /// where they differ from Annex 1's (`MinimumAges`), not here.
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

    fn matches(&self, id: &LicenceTitleId) -> bool {
        match (self, id) {
            (Self::Annex1 { title, .. }, LicenceTitleId::Annex1(other)) => title == other,
            (Self::National { term, .. }, LicenceTitleId::National(other)) => &term.id == other,
            _ => false,
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

/// A title held under a licence, from the date the holder was first issued
/// it: the date of first issue, not of the latest document, so a reissued
/// document does not move it.
///
/// There is no expiry or suspension to record yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldLicenceTitle {
    pub title: LicenceTitleId,
    pub date_of_first_issue: Date,
}

impl HeldLicenceTitle {
    pub fn new(title: LicenceTitleId, date_of_first_issue: Date) -> Self {
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

/// A rule the titles held under a licence break, for any authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenceTitleProblem {
    /// The authority issues no such title.
    NotIssued(LicenceTitleId),
    /// The title is listed more than once under the licence, so it carries
    /// more than one date of first issue.
    Repeated(LicenceTitleId),
    /// The holder had not reached the title's minimum age on its date of
    /// first issue. Only the authority's own titles are checked here: an
    /// Annex 1 title's age is checked by icao-shared-kernel's `Pilot`, with
    /// the State's ages where they differ from Annex 1's.
    BeforeMinimumAge {
        title: LicenceTitleId,
        date_of_first_issue: Date,
        minimum_age: u8,
    },
}

/// The problems with `held`, the titles listed under one licence, against the
/// titles its authority issues, for a holder born on `date_of_birth`, in the
/// order the titles are listed. [`LicensingAuthority::licence_title_problems`]
/// is this over the authority's own titles.
///
/// Only the authority's own titles' ages are checked here, so this is not the
/// whole age check: an Annex 1 title's is icao-shared-kernel's `Pilot`'s, with
/// the State's ages where they differ from Annex 1's. A title listed more
/// than once is reported as repeated once, and each listing's age is checked.
///
/// A year is complete on the birthday, compared by month and day: someone born
/// on 29 February completes a year on 1 March in a year that is not a leap
/// year. The age is checked on the date of first issue, so it can miss an
/// applicant who was too young when they applied, though not one who was too
/// young when the title was issued.
///
/// [`LicensingAuthority::licence_title_problems`]: super::LicensingAuthority::licence_title_problems
pub fn licence_title_problems(
    issued: &[LicenceTitleTerm],
    held: &[HeldLicenceTitle],
    date_of_birth: Date,
) -> Vec<LicenceTitleProblem> {
    let mut seen = BTreeSet::new();
    let mut reported = BTreeSet::new();
    let mut problems = Vec::new();
    for title in held {
        let first = seen.insert(&title.title);
        if !first && reported.insert(&title.title) {
            problems.push(LicenceTitleProblem::Repeated(title.title.clone()));
        }
        match issued.iter().find(|term| term.matches(&title.title)) {
            None if first => problems.push(LicenceTitleProblem::NotIssued(title.title.clone())),
            Some(LicenceTitleTerm::National { minimum_age, .. })
                if age_on(date_of_birth, title.date_of_first_issue) < i32::from(*minimum_age) =>
            {
                problems.push(LicenceTitleProblem::BeforeMinimumAge {
                    title: title.title.clone(),
                    date_of_first_issue: title.date_of_first_issue,
                    minimum_age: *minimum_age,
                });
            }
            _ => {}
        }
    }
    problems
}

/// Whole years completed on `on` by someone born on `born`; negative before
/// the birth. A copy of icao-shared-kernel's, which does not export it.
fn age_on(born: Date, on: Date) -> i32 {
    let years = on.year() - born.year();
    if (u8::from(on.month()), on.day()) < (u8::from(born.month()), born.day()) {
        years - 1
    } else {
        years
    }
}
