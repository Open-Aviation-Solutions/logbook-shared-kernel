// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The result of a currency check, and the traits a check implements.

use time::{Date, Duration};

use super::accumulator::{AssessmentAccumulator, EntryAccumulator};
use crate::entry::{AssessmentId, EntryId};

/// An entry that counted towards a check, and how many of the check's events
/// it supplied (take-off and landing pairs, approaches, qualifying flights).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contribution {
    pub entry: EntryId,
    /// As recorded, saturating at `usize::MAX` (32 bits on `wasm32`).
    pub events: usize,
}

/// Outcome of a single currency check.
///
/// `expiry_date` is computed whenever at least `required` qualifying events
/// have ever occurred, regardless of the look-back window, so a caller can
/// report both "expires on X" (future) and "expired on X" (past). It is `None`
/// only when the pilot has never accumulated enough qualifying events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyResult {
    pub expiry_date: Option<Date>,
    /// The as-of date the check was built for.
    pub reference: Date,
    /// Entries inside the look-back window that had qualifying events, with
    /// how many each supplied, in the order of their ids, so that the result
    /// does not depend on the order the entries arrived in.
    pub qualifying: Vec<Contribution>,
    /// The same, for the checks driven by assessments rather than entries,
    /// in the order of their ids.
    pub qualifying_assessments: Vec<AssessmentId>,
    /// Qualifying events found inside the window, as recorded, saturating at
    /// `usize::MAX` (32 bits on `wasm32`).
    pub found: usize,
    /// Events required.
    pub required: usize,
    /// How far back, in days, `qualifying` looks.
    pub look_back_days: i64,
}

impl CurrencyResult {
    pub fn is_current(&self) -> bool {
        self.expiry_date
            .is_some_and(|expiry| expiry > self.reference)
    }

    /// The last day the pilot is (or was) current: the day before
    /// `expiry_date`, which is the first day they are not. `None` when they
    /// never have been.
    pub fn current_through(&self) -> Option<Date> {
        self.expiry_date
            .and_then(|expiry| expiry.checked_sub(Duration::days(1)))
    }

    /// Further qualifying events needed to become current.
    pub fn needed(&self) -> usize {
        self.required.saturating_sub(self.found)
    }
}

/// An accumulator over entries that answers a currency question.
pub trait CurrencyCheck: EntryAccumulator {
    fn result(&self) -> CurrencyResult;
}

/// An accumulator over assessments that answers a currency question.
pub trait AssessmentCurrencyCheck: AssessmentAccumulator {
    fn result(&self) -> CurrencyResult;
}
