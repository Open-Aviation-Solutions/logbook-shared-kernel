// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The logbook's shared kernel: the part of a pilot's logbook that the
//! logbook and each licensing authority's rules agree on.
//!
//! It holds, in two modules:
//!
//! - [`entry`]: the logbook's common entry, in the terms ICAO's Annex 1
//!   defines, and an authority's section of it, which is the authority's
//!   and holds the answers to its form, each under its question's key.
//! - [`authority`]: the trait an authority implements to supply its rules to
//!   the logbook, the machinery its currency checks are written over, and
//!   the form description, in which it asks its questions about an entry.
//!
//! The design is recorded in pilot-logbook's ADR 0001.

pub mod authority;
pub mod entry;

pub use authority::{
    run, run_assessments, Answer, AssessmentAccumulator, AssessmentCurrencyCheck, Choice,
    Classification, Condition, Contribution, CurrencyCheck, CurrencyResult, DateRange,
    EntryAccumulator, EventWindow, Figure, Form, LicensingAuthority, Measure, Problem, ProblemKind,
    Question, Requirement, RequirementId, Total,
};

pub use entry::{
    AircraftDescription, Approach, ApproachError, ApproachType, Assessment, AssessmentId,
    AuthorityId, DayNight, Entry, EntryError, EntryId, EntryParts, InstrumentTime, Logbook, Route,
    Section, Subject,
};
