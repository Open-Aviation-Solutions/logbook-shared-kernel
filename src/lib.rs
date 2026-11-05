// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The logbook's shared kernel: the part of a pilot's logbook that the
//! logbook and each licensing authority's rules agree on.
//!
//! It is to hold, in two modules:
//!
//! - [`entry`]: the logbook's common entry, in the terms ICAO's Annex 1
//!   defines, and an authority's section of it, which only that authority
//!   reads.
//! - [`authority`]: the trait an authority implements to supply its rules to
//!   the logbook, the machinery its currency checks are written over, and
//!   the form description, in which it asks its questions about an entry.
//!
//! `authority` holds the machinery so far; the trait and the form
//! description follow before the first release, 0.1.0 (issue #1). The design
//! is recorded in pilot-logbook's ADR 0001.

pub mod authority;
pub mod entry;

pub use authority::{
    run_assessments, AssessmentAccumulator, AssessmentCurrencyCheck, Contribution, CurrencyCheck,
    CurrencyResult, DateRange, EntryAccumulator, EventWindow, Measure, Total,
};

pub use entry::{
    AircraftDescription, Approach, ApproachType, Assessment, AssessmentId, AuthorityId, DayNight,
    Entry, EntryError, EntryId, EntryParts, InstrumentTime, Logbook, Route, Section, Subject,
};
