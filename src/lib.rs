// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The logbook's shared kernel: the part of a pilot's logbook that the
//! logbook and each licensing authority's rules agree on.
//!
//! It holds, in two modules:
//!
//! - [`entry`]: the logbook's common entry, in the terms ICAO's Annex 1
//!   defines, and an authority's information on it or on an assessment,
//!   which is the authority's and holds the answers to its questions, each
//!   under its question's key.
//! - [`authority`]: the trait an authority implements to supply its rules to
//!   the logbook, the machinery its currency checks are written over, and
//!   the description of the questions it asks about an entry or an
//!   assessment.
//!
//! The design is recorded in pilot-logbook's ADR 0001.

pub mod authority;
pub mod entry;

pub use authority::{
    run, run_assessments, Answer, AssessmentAccumulator, AssessmentCurrencyCheck, Choice,
    ChoiceGroup, ChoiceKind, Classification, Condition, Contribution, CurrencyCheck,
    CurrencyResult, DateRange, EntryAccumulator, EventWindow, Figure, LicensingAuthority, Measure,
    Problem, ProblemAt, ProblemKind, Question, Questions, Requirement, RequirementId, Total,
};

pub use entry::{
    AircraftDescription, Approach, ApproachType, Assessment, AssessmentId, AuthorityId,
    AuthoritySpecificInformation, DayNight, Entry, EntryError, EntryId, EntryParts, InstrumentTime,
    Logbook, Record, RnpApchMinima, Route, Subject, SubjectId,
};
