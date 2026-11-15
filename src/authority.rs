// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The interface through which a licensing authority supplies its rules to
//! the logbook, and what those rules are written over: the accumulators that
//! fold a logbook into a result in one pass, the currency checks and their
//! event window, the totals, and the description of an authority's
//! questions.
//!
//! Every name, label and text an authority fills in here is its own, in its
//! regulation's words: only the authority knows them.

mod accumulator;
mod currency;
mod event_window;
mod licence_titles;
mod questions;
mod totals;
pub mod window;

use std::collections::BTreeSet;

use icao_shared_kernel::FlightDuration;
use time::Date;

use crate::entry::{
    AircraftCharacteristic, AircraftDescription, AuthorityId, AuthoritySpecificInformation, Entry,
    Logbook, Record,
};

pub use accumulator::{run, run_assessments, AssessmentAccumulator, EntryAccumulator};
pub use currency::{AssessmentCurrencyCheck, Contribution, CurrencyCheck, CurrencyResult};
pub use event_window::EventWindow;
pub use licence_titles::{
    licence_title_problems, HeldLicenceTitle, LicenceTitleId, LicenceTitleProblem, LicenceTitleTerm,
};
pub use questions::{
    Answer, Choice, ChoiceGroup, ChoiceKind, ChoicesLabels, Condition, Question, Questions,
};
pub use totals::{DateRange, Measure, Total};

/// A licensing authority's rules, as the logbook calls them, for one
/// authority.
///
/// Object-safe, so a host holds `Box<dyn LicensingAuthority>`, one for each
/// authority the pilot holds a licence from, found by [`id`]. That is why
/// authority-specific information crosses this interface as JSON and not as
/// an associated type. Rules that several authorities issue are built once
/// for each authority, each object holding its authority's id.
///
/// [`id`]: LicensingAuthority::id
pub trait LicensingAuthority {
    /// The authority whose licences these rules serve, and whose
    /// information on a record they read and check.
    fn id(&self) -> &AuthorityId;

    /// Whether an entry counts under these rules at all. [`run`] applies it
    /// once, before any accumulator sees an entry.
    fn counts(&self, entry: &Entry) -> bool;

    /// What an aircraft counts as under these rules, and what they require
    /// to fly it, from its type's description and its airframe's
    /// characteristics. `None`: it cannot say.
    ///
    /// A characteristic that the aircraft's category does not use (skis on
    /// a helicopter, say) is not a problem: it is a physical fact, and the
    /// rules simply require nothing for it.
    ///
    /// A session carries no characteristics, so a session's simulated type
    /// is classified with an empty set.
    fn classify(
        &self,
        aircraft: &AircraftDescription,
        characteristics: &BTreeSet<AircraftCharacteristic>,
    ) -> Option<Classification>;

    /// Every title of licence the authority issues, of either kind: an Annex 1
    /// title under the authority's own term for it, or one of its own. An
    /// Annex 1 title left out is one the authority does not issue.
    ///
    /// What a host may rely on, which the authority keeps and nothing checks:
    /// no title is listed twice, and no two titles, Annex 1's and its own alike,
    /// share a term id.
    fn licence_titles(&self) -> Vec<LicenceTitleTerm>;

    /// The problems with the titles held under one licence of this
    /// authority's, for a holder born on `date_of_birth`: [`licence_title_problems`]
    /// over [`licence_titles`](LicensingAuthority::licence_titles).
    fn licence_title_problems(
        &self,
        held: &[HeldLicenceTitle],
        date_of_birth: Date,
    ) -> Vec<LicenceTitleProblem> {
        licence_title_problems(&self.licence_titles(), held, date_of_birth)
    }

    /// The requirements these rules check.
    fn requirements(&self) -> Vec<Requirement>;

    /// Each requirement's result on a day.
    fn currency(&self, as_of: Date, logbook: &Logbook) -> Vec<(RequirementId, CurrencyResult)>;

    /// The authority's own totals of the entries dated in `range`.
    fn totals(&self, logbook: &Logbook, range: DateRange) -> Vec<AuthorityTotal>;

    /// The questions the authority asks for its information on `record`,
    /// an entry or an assessment. An authority with nothing to ask about a
    /// kind of record returns no questions for it.
    ///
    /// A question that is asked only of some records (only of a session,
    /// say, or only of an entry with an approach) is kept in the list, with
    /// a [`Condition`] on the record as its `applies_when`, rather than
    /// left out. Then [`Questions::retain_applicable_answers_for`]
    /// removes its answer once the record no longer meets the condition;
    /// an answer to a question left out would be kept, as no question's.
    fn questions_for(&self, record: Record) -> Questions;

    /// The problems with `information`, the authority's information on
    /// `record`, each at a question or at the information as a whole
    /// ([`ProblemAt`]): a missing or unsupported version is reported at the
    /// information as a whole. The record gives the context (an entry's
    /// capacity, say); the information is what is checked, and is the one
    /// passed, not the record's own.
    ///
    /// The host saves a record's information in this order: it finds the
    /// stored information ([`Record::information_for`] the authority's
    /// [`id`]) or starts new; it removes the answers to questions that do
    /// not apply ([`Questions::retain_applicable_answers_for`]); it asks for
    /// the problems with what is left, here, and for the required questions
    /// left unanswered ([`Questions::required_unanswered_for`]); and, unless a
    /// problem is a refusal, it writes the version ([`Questions::stamp`]) and
    /// saves. So the information's version is as it was stored
    /// ([`AuthoritySpecificInformation::version`]): `None` for information
    /// not saved yet.
    ///
    /// An authority that asks nothing about a kind of record returns no
    /// questions for it, and so reports no problem at a question.
    ///
    /// [`AuthoritySpecificInformation::version`]: crate::entry::AuthoritySpecificInformation::version
    /// [`Record::information_for`]: crate::entry::Record::information_for
    /// [`id`]: LicensingAuthority::id
    fn problems_for(
        &self,
        record: Record,
        information: &AuthoritySpecificInformation,
    ) -> Vec<Problem>;
}

/// Identifies a requirement. The authority's, and opaque: one regulation
/// can set several (a flight review for each class rating, say).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequirementId(String);

impl RequirementId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A requirement an authority checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub id: RequirementId,
    /// Its name, in the regulation's words.
    pub name: String,
    /// The regulation that sets it.
    pub regulation: &'static str,
}

/// Identifies one of an authority's terms: a category or class an aircraft
/// counts as, something it requires to fly one, one of its totals, or a title of licence. The
/// authority's, stable, and opaque: the host matches on it, never on a
/// term's name, so the authority can reword a name freely. Classification
/// and totals share it, so a total can be identified as one of the
/// authority's categories, say.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TermId(String);

impl TermId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One of an authority's terms, identified by its id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub id: TermId,
    /// Its name, in the authority's words, for display only.
    pub name: String,
}

/// What an aircraft counts as under an authority's rules, and what they
/// require to fly it, in the authority's terms. The host only shows it: the
/// checks that need it are the authority's own.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Classification {
    /// Its category and class (CASR 1998 regs 61.015 and 61.020, say).
    pub counts_as: Vec<Term>,
    /// What the authority requires for the aircraft beyond its category
    /// and class: a design feature endorsement (CASR 1998 reg 61.755), say.
    pub requires: Vec<Term>,
    pub cautions: Vec<String>,
}

/// A problem with an authority's information on a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub at: ProblemAt,
    pub kind: ProblemKind,
    pub text: String,
    pub regulation: &'static str,
}

/// Where in the authority-specific information a problem is reported, for
/// the host to show it there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProblemAt {
    /// At the question with this key ([`Question::key`]).
    Question(&'static str),
    /// At the information as a whole, such as a missing or unsupported
    /// version ([`AuthoritySpecificInformation::version`]). The host shows it
    /// at the top of the authority's questions.
    ///
    /// [`AuthoritySpecificInformation::version`]: crate::entry::AuthoritySpecificInformation::version
    Information,
}

/// Whether a problem stops the record (an entry or an assessment) being
/// saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProblemKind {
    /// The authority cannot accept the information: it contradicts itself,
    /// or its version is missing or unsupported. The record is not saved.
    Refusal,
    /// The information says what the authority's conditions do not
    /// support. The record is the pilot's claim, so it is saved, and the
    /// host shows the warning.
    Warning,
}

/// One of an authority's own totals, in a flat list in which each says the
/// total it is part of.
///
/// What a host may rely on in the list [`LicensingAuthority::totals`]
/// returns, which the authority keeps and nothing checks: each `id` is
/// unique within the list, and each `part_of` names another total in the
/// same list, which comes before it, so the totals form no cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityTotal {
    pub id: TermId,
    /// Its name, in the authority's words, for display only.
    pub name: String,
    pub time: FlightDuration,
    /// The total it is part of: a class of CASR 1998 reg 61.020 under its
    /// category of reg 61.015, say. `None`: it is part of no other.
    pub part_of: Option<TermId>,
}
