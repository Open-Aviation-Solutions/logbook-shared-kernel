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
mod migration;
mod questions;
mod totals;
pub mod window;

use std::collections::BTreeSet;

use icao_shared_kernel::{FlightDuration, Licence, MinimumAges};
use time::Date;

use crate::entry::{
    AircraftCharacteristic, AircraftDescription, Assessment, AuthorityId,
    AuthoritySpecificInformation, Entry, Logbook, Record,
};

pub use accumulator::{run, run_assessments, AssessmentAccumulator, EntryAccumulator};
pub use currency::{AssessmentCurrencyCheck, Contribution, CurrencyCheck, CurrencyResult, Expiry};
pub use event_window::EventWindow;
pub use licence_titles::{
    licence_title_problems, HeldLicenceTitle, LicenceTitleId, LicenceTitleProblem, LicenceTitleTerm,
};
pub use migration::MigrationRefused;
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
    /// A session's simulated type is classified with the characteristics
    /// recorded for the session ([`Simulated::characteristics`](crate::entry::Simulated::characteristics)).
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

    /// Every rating and endorsement the authority issues, each by its id and
    /// its name in the authority's words, in the authority's order. A host
    /// may match the ones a pilot holds against what an aircraft requires
    /// ([`Classification::requires`]) by equal ids, to show whether they are
    /// held; the checks stay the authority's own.
    ///
    /// What a host may rely on, which the authority keeps and nothing checks:
    /// no id is listed twice, no id is also a title of licence's
    /// ([`licence_titles`](LicensingAuthority::licence_titles)), and a
    /// [`TermId`] that `requires` gives is one of these.
    fn ratings_and_endorsements(&self) -> Vec<Term>;

    /// The minimum age at which each Annex 1 title is first issued, under
    /// these rules: required, so an authority states its ages even where they
    /// are Annex 1's ([`MinimumAges::default`]). A host builds the
    /// [`Pilot`](icao_shared_kernel::Pilot) with
    /// `Pilot::with_minimum_ages` (or `Pilot::deserialize_with`), giving it a
    /// function from each licence's issuing State and authority to that
    /// authority's `minimum_ages()`; the ages of each licence are then
    /// checked against its own authority's. That checks the Annex 1 titles'
    /// ages (`Licence::new` refuses a title listed twice). The own titles'
    /// ages are in [`licence_titles`](LicensingAuthority::licence_titles).
    fn minimum_ages(&self) -> MinimumAges;

    /// The problems with the titles under one licence of this authority's, for
    /// a holder born on `date_of_birth`, `own` being the authority's own titles
    /// held under it. `licence` must be one this authority issued
    /// ([`AuthorityId::of`] the licence is [`id`](LicensingAuthority::id)):
    /// it is not checked, and another authority's would be checked against
    /// these titles. [`licence_title_problems`] over
    /// [`licence_titles`](LicensingAuthority::licence_titles).
    fn licence_title_problems(
        &self,
        licence: &Licence,
        own: &[HeldLicenceTitle],
        date_of_birth: Date,
    ) -> Vec<LicenceTitleProblem> {
        licence_title_problems(&self.licence_titles(), licence, own, date_of_birth)
    }

    /// The problems with what is held under one licence of this authority's:
    /// `titles` are its titles (Annex 1's and the authority's own, as
    /// [`LicenceTitleId`]s) and `ratings_and_endorsements` the ids of the
    /// ratings and endorsements held under it. Each problem is a refusal or a
    /// warning ([`ProblemKind`]). As for
    /// [`licence_title_problems`](LicensingAuthority::licence_title_problems),
    /// the licence must be one this authority issued, which is not checked.
    ///
    /// What a host may rely on, which the authority keeps and nothing checks:
    /// each problem's `at` is one of `titles` or `ratings_and_endorsements`,
    /// or [`LicenceProblemAt::Licence`], and something missing is reported at
    /// `Licence`.
    ///
    /// None by default; it is separate from the titles' own structural
    /// problems, which `licence_title_problems` reports.
    fn licence_problems(
        &self,
        titles: &[LicenceTitleId],
        ratings_and_endorsements: &[TermId],
    ) -> Vec<LicenceProblem> {
        let _ = (titles, ratings_and_endorsements);
        Vec::new()
    }

    /// What a passed flight test on `assessment` would have granted, by the
    /// ids the authority lists them under: a title of licence by
    /// [`licence_titles`](LicensingAuthority::licence_titles) and a rating or
    /// endorsement by
    /// [`ratings_and_endorsements`](LicensingAuthority::ratings_and_endorsements).
    /// `entry` is the entry the assessment names, and `classification` is this
    /// authority's own classification of that entry's aircraft (what
    /// [`classify`](LicensingAuthority::classify) gave for it), since a purpose
    /// may not name its grant fully: a title for the category of the flight's
    /// aircraft, say. The host passes `None` when `classify` gave `None`.
    ///
    /// Only a passed flight test grants: none for any other kind of
    /// assessment (a flight review or proficiency check renews what is held,
    /// it does not grant it), and none for a purpose that was failed. The
    /// authority reads its own information on the assessment to tell.
    ///
    /// What the host keeps, which nothing checks: `entry` is the entry the
    /// assessment names, and `classification` is what `classify` gave for that
    /// entry's aircraft.
    ///
    /// What a host may rely on, which the authority keeps and nothing checks:
    /// each [`Grant`] is an id the authority lists, as a title or as a rating
    /// or endorsement respectively, and none is given twice. With no
    /// classification the authority still gives the grants it can name
    /// without one, and leaves out only those that need one.
    ///
    /// None by default.
    fn grants_from(
        &self,
        assessment: &Assessment,
        entry: &Entry,
        classification: Option<&Classification>,
    ) -> Vec<Grant> {
        let _ = (assessment, entry, classification);
        Vec::new()
    }

    /// The requirements these rules check, each with its id, its name in the
    /// regulation's words, a description of what meets it, the noun its
    /// events are counted in, the set it belongs to (with that set's name and
    /// description, in the authority's words), its kind, and the regulation it
    /// is set by. Every one of them, whatever a pilot holds:
    /// [`requirements_for`](LicensingAuthority::requirements_for) answers
    /// those that apply to a pilot.
    fn requirements(&self) -> Vec<Requirement>;

    /// The requirements that apply to a pilot who holds `held`, in the order
    /// of [`requirements`](LicensingAuthority::requirements). A requirement
    /// applies only for something held, so nothing held answers none.
    ///
    /// `held` is the id of each title of licence, rating and endorsement the
    /// pilot holds under the licences this authority issued, on the day the
    /// host asks about. An Annex 1 title (one of `Licence::titles`) is held
    /// by the id of the authority's term for it: `term().id` of the
    /// [`LicenceTitleTerm`] for that title in
    /// [`licence_titles`](LicensingAuthority::licence_titles). An own title
    /// ([`HeldLicenceTitle::title`]) and a rating or endorsement are held by
    /// their ids as they are, which are the authority's already. The host
    /// chooses the day, and leaves out what was first issued after it. An id
    /// that is wrong is not reported: it makes fewer requirements apply.
    /// What a requirement applies to is the authority's own rule (a
    /// requirement for one category of aircraft, say, may apply to any title
    /// or rating in that category); an id the authority does not issue makes
    /// nothing apply.
    ///
    /// What a host may rely on, which the authority keeps and nothing checks:
    /// each requirement answered is one of `requirements`, answered once; and
    /// the members of a set apply together, so a set is answered whole or not
    /// at all, never with some of its members.
    fn requirements_for(&self, held: &BTreeSet<TermId>) -> Vec<Requirement>;

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
    /// capacity, say, or an assessment's aircraft, from the entry it names,
    /// given with it); the information is what is checked, and is
    /// the one passed, not the record's own.
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
    /// The host also asks again about each assessment that names an entry
    /// whenever that entry is changed, with the entry as it would be and the
    /// assessment's stored information, for each authority that holds
    /// information on it; it refuses the change when a problem with any of
    /// them is a refusal. It does not delete an entry that an assessment
    /// names.
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

    /// The authority's stored information on `record`, brought up to the
    /// version these rules write, from any older version they still read:
    /// `Ok(None)` when it is at that version already, so there is nothing to
    /// rewrite. `Ok(None)` says only that: it is not a check of the answers,
    /// which is [`problems_for`]'s.
    ///
    /// Only stored information is migrated. A record with no information for
    /// this authority ([`Record::information_for`] gives `None`) has nothing
    /// to migrate, and the host does not call this for it: its questions are
    /// unanswered, which is [`problems_for`]'s to report. Stored information
    /// always holds a version, as the host stamps it on saving, so stored
    /// information without one is damaged.
    ///
    /// The migrated information is stamped with the current version
    /// ([`Questions::stamp`]) and reads as current to this authority's own
    /// [`problems_for`]. Running it on what it returned gives `Ok(None)`.
    /// A migration changes only what it guesses nothing about.
    ///
    /// Information these rules cannot migrate is refused
    /// ([`MigrationRefused`]), by kind, and the host's advice follows the
    /// kind: from a newer version than these rules write ([`Newer`]), update
    /// the app; older than every migration the authority keeps
    /// ([`TooOld`]), update in steps; not readable as the version it claims
    /// ([`Unreadable`]), including stored information with no version, the
    /// record is damaged and its file needs correcting.
    ///
    /// The default is for an authority with no format history: it reads the
    /// version only, returning `Ok(None)` for information at the version of
    /// [`questions_for`](LicensingAuthority::questions_for) for the record's
    /// kind, and refuses any other, so an authority whose information has versions and has not
    /// implemented this refuses its older information visibly and does not
    /// pass it as current.
    ///
    /// [`problems_for`]: LicensingAuthority::problems_for
    /// [`Newer`]: MigrationRefused::Newer
    /// [`TooOld`]: MigrationRefused::TooOld
    /// [`Unreadable`]: MigrationRefused::Unreadable
    /// [`Record::information_for`]: crate::entry::Record::information_for
    fn migrate_information(
        &self,
        record: Record,
        information: &AuthoritySpecificInformation,
    ) -> Result<Option<AuthoritySpecificInformation>, MigrationRefused> {
        let current = self.questions_for(record).version();
        match information.version() {
            Some(version) if version == current => Ok(None),
            Some(version) if version > current => Err(MigrationRefused::Newer(format!(
                "the information is of version {version}, newer than version {current}, which these rules write"
            ))),
            Some(version) => Err(MigrationRefused::TooOld(format!(
                "the information is of version {version}, which these rules have no migration from, to version {current}"
            ))),
            None => Err(MigrationRefused::Unreadable(format!(
                "the information holds no version: a JSON object with \"{}\" a whole number is expected",
                AuthoritySpecificInformation::VERSION_KEY
            ))),
        }
    }
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

/// A requirement an authority checks, with everything a host shows of it in
/// the authority's words, so that the host holds no wording or grouping keyed
/// by an authority's ids or regulations. The host words only a heading for
/// each [`RequirementKind`], which is the kernel's.
///
/// Non-exhaustive, and built with [`Requirement::new`], so that a field added
/// later is not a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Requirement {
    pub id: RequirementId,
    /// Its name, in the regulation's words.
    pub name: String,
    /// What meets it, in the authority's words: "1 night landing in the past
    /// 6 months", say.
    pub description: String,
    /// The noun its events are counted in, for a requirement met by a count
    /// of events. The authority's checks count them
    /// ([`CurrencyResult`]); the host shows a count with this noun.
    pub events: Option<EventsNoun>,
    /// The set a regulation sets it together with others in, so that the host
    /// shows the set as one check without reading the members' ids or names.
    /// Each member keeps its own name, description and events, so that the
    /// host can say which is short. Members sharing a set's `key` carry equal
    /// sets (the same name and description), which the authority keeps and
    /// nothing checks. `None`: it is set on its own.
    pub set: Option<RequirementSet>,
    /// Whether it is recent experience or a periodic assessment, which the
    /// host groups requirements under for its headings.
    pub kind: RequirementKind,
    /// The regulation that sets it.
    pub regulation: &'static str,
}

impl Requirement {
    /// A requirement with no events noun and set on its own; add those with
    /// [`with_events`](Requirement::with_events) and
    /// [`with_set`](Requirement::with_set).
    pub fn new(
        id: RequirementId,
        name: impl Into<String>,
        description: impl Into<String>,
        kind: RequirementKind,
        regulation: &'static str,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            events: None,
            set: None,
            kind,
            regulation,
        }
    }

    /// The requirement, with its events counted in `events`.
    pub fn with_events(self, events: EventsNoun) -> Self {
        Self {
            events: Some(events),
            ..self
        }
    }

    /// The requirement, set together with the others in `set`.
    pub fn with_set(self, set: RequirementSet) -> Self {
        Self {
            set: Some(set),
            ..self
        }
    }
}

/// What a host shows of a set of requirements a regulation sets together, in
/// the authority's words: one check that is met when every member is.
///
/// Non-exhaustive, and built with [`RequirementSet::new`], so that a field
/// added later is not a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RequirementSet {
    /// The key its members share. The authority's, and opaque.
    pub key: String,
    /// Its name: "Carrying a passenger by day in an aeroplane", say.
    pub name: String,
    /// What meets it: "At least 3 take-offs and 3 landings, by day or night,
    /// in the past 90 days", say.
    pub description: String,
}

impl RequirementSet {
    /// A set from its `key`, its `name` and its `description`, in that order.
    pub fn new(
        key: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            description: description.into(),
        }
    }
}

/// The noun a requirement's events are counted in, in both of its forms, so
/// that no host makes a plural: "take-off" and "take-offs", say. The fields
/// are named for the plural categories they are used in (one take-off, any
/// other number of take-offs). A language with more forms would need a form
/// added, which is a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventsNoun {
    /// The form for one event.
    pub one: String,
    /// The form for any other number of events.
    pub other: String,
}

/// The kind of a requirement, which holds for every authority's: the host
/// words a heading for each kind. It groups requirements for those headings
/// and says nothing about what meets one: an assessment can meet a recent
/// experience requirement too.
///
/// Non-exhaustive, so that a kind an authority needs later is not a breaking
/// change: a host lists a kind it does not know under a heading of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RequirementKind {
    /// Recent experience: what the pilot must have done in a past period to
    /// exercise a privilege (CASR 1998 reg 61.395, say).
    RecentExperience,
    /// A periodic assessment: the kernel's term for the requirements Part 61
    /// meets with a flight review or a proficiency check, named for the
    /// kernel's [`Assessment`](crate::entry::Assessment), which records one,
    /// as decided on logbook-shared-kernel#46. No regulation uses the term.
    PeriodicAssessment,
}

/// Identifies one of an authority's terms: a category or class an aircraft
/// counts as, something it requires to fly one, one of its totals, a title of licence, or a rating
/// or endorsement. The
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
/// require to fly it, in the authority's terms. The host only shows it, and may match what
/// is required against the ratings and endorsements a pilot holds to show
/// whether they are held: the checks that need it are the authority's own.
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

/// What a passed flight test grants ([`LicensingAuthority::grants_from`]).
/// The ids are the authority's own, which the host matches by equality and
/// never parses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Grant {
    /// A title of licence: Annex 1's, or the authority's own.
    LicenceTitle(LicenceTitleId),
    /// A rating or endorsement.
    RatingOrEndorsement(TermId),
}

/// A problem with what is held under a licence: a refusal, which stops the
/// licence being saved, or a warning, which the host shows. Built with
/// [`LicenceProblem::new`], so that a field may be added without breaking an
/// authority that gives these.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct LicenceProblem {
    pub at: LicenceProblemAt,
    pub kind: ProblemKind,
    /// In the authority's words.
    pub text: String,
    pub regulation: &'static str,
}

impl LicenceProblem {
    pub fn new(
        at: LicenceProblemAt,
        kind: ProblemKind,
        text: impl Into<String>,
        regulation: &'static str,
    ) -> Self {
        Self {
            at,
            kind,
            text: text.into(),
            regulation,
        }
    }
}

/// Where under a licence a [`LicenceProblem`] is reported, for the host to
/// show it there.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LicenceProblemAt {
    /// At a title held under the licence.
    Title(LicenceTitleId),
    /// At a rating or endorsement held under it.
    RatingOrEndorsement(TermId),
    /// At the licence as a whole: something missing from it, say.
    Licence,
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

/// Whether a problem stops what it is about being saved: a record (an entry
/// or an assessment) or what is held under a licence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProblemKind {
    /// The authority cannot accept it: the information contradicts itself,
    /// or its version is missing or unsupported, or what is held under a
    /// licence breaks a rule. It is not saved.
    Refusal,
    /// It says what the authority's conditions do not support. It is the
    /// pilot's claim, so it is saved, and the host shows the warning.
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
