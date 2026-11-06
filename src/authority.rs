// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The interface through which a licensing authority supplies its rules to
//! the logbook, and what those rules are written over: the accumulators that
//! fold a logbook into a result in one pass, the currency checks and their
//! event window, the totals, and the form description.
//!
//! Every name, label and text an authority fills in here is its own, in its
//! regulation's words: only the authority knows them.

mod accumulator;
mod currency;
mod event_window;
mod form;
mod totals;
pub mod window;

use icao_shared_kernel::FlightDuration;
use time::Date;

use crate::entry::{AircraftDescription, AuthorityId, Entry, Logbook};

pub use accumulator::{run, run_assessments, AssessmentAccumulator, EntryAccumulator};
pub use currency::{AssessmentCurrencyCheck, Contribution, CurrencyCheck, CurrencyResult};
pub use event_window::EventWindow;
pub use form::{Answer, Choice, Condition, Form, Question};
pub use totals::{DateRange, Measure, Total};

/// A licensing authority's rules, as the logbook calls them.
///
/// Object-safe, so a host holds `Box<dyn LicensingAuthority>`, one for each
/// authority the pilot holds a licence from, found by [`authorities`]. That
/// is why a section crosses this interface as JSON and not as an associated
/// type.
///
/// [`authorities`]: LicensingAuthority::authorities
pub trait LicensingAuthority {
    /// The authorities whose licences these rules serve: one, or several
    /// when one State's rules are issued by several authorities.
    fn authorities(&self) -> &[AuthorityId];

    /// Whether an entry counts under these rules at all. [`run`] applies it
    /// once, before any accumulator sees an entry.
    fn counts(&self, entry: &Entry) -> bool;

    /// What an aircraft type counts as under these rules, from its
    /// description. `None`: it cannot say.
    fn classify(&self, aircraft: &AircraftDescription) -> Option<Classification>;

    /// The requirements these rules check.
    fn requirements(&self) -> Vec<Requirement>;

    /// Each requirement's result on a day.
    fn currency(&self, as_of: Date, logbook: &Logbook) -> Vec<(RequirementId, CurrencyResult)>;

    /// The authority's own totals of the entries dated in `range`.
    fn totals(&self, logbook: &Logbook, range: DateRange) -> Vec<Figure>;

    /// The questions `authority`, one of [`authorities`], asks about an
    /// entry for its section, given the entry's common part.
    ///
    /// [`authorities`]: LicensingAuthority::authorities
    fn section_form(&self, authority: &AuthorityId, entry: &Entry) -> Form;

    /// The problems with `authority`'s section of an entry, each at a
    /// question or at the section as a whole ([`ProblemAt`]): a missing or
    /// unsupported version is reported at the section. `authority` is one
    /// of [`authorities`], so the host knows
    /// which section each problem is in. The host has removed the answers
    /// to hidden questions first ([`Form::retain_shown`]), and
    /// [`Form::unanswered`] lists the required questions left unanswered.
    /// The section's version is as it was stored ([`Section::version`]):
    /// `None` for a section not saved yet, since the host writes the form's
    /// version only after this, as it saves the section ([`Form::stamp`]).
    ///
    /// [`Section::version`]: crate::entry::Section::version
    ///
    /// [`authorities`]: LicensingAuthority::authorities
    fn validate_section(&self, authority: &AuthorityId, entry: &Entry) -> Vec<Problem>;
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

/// What an aircraft type counts as under an authority's rules, in its
/// terms. The host only shows it: the checks that need it are the
/// authority's own.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Classification {
    pub counts_as: Vec<String>,
    pub cautions: Vec<String>,
}

/// A problem with an authority's section of an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub at: ProblemAt,
    pub kind: ProblemKind,
    pub text: String,
    pub regulation: &'static str,
}

/// Where in the section a problem is reported, for the host to show it there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProblemAt {
    /// At the question with this key ([`Question::key`]).
    Question(&'static str),
    /// At the section as a whole, such as a missing or unsupported version
    /// ([`Section::version`]). The host shows it at the top of the section.
    ///
    /// [`Section::version`]: crate::entry::Section::version
    Section,
}

/// Whether a problem stops the entry being saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProblemKind {
    /// The section contradicts itself, and the entry is not saved.
    Refusal,
    /// The section says what the authority's conditions do not support. The
    /// entry is the pilot's claim, so it is saved, and the host shows the
    /// warning.
    Warning,
}

/// One of an authority's own totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Figure {
    /// Its name, in the authority's words.
    pub name: String,
    pub time: FlightDuration,
}
