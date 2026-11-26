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

/// When a currency ends ([`CurrencyResult::expiry`]), or the last day of it
/// ([`CurrencyResult::current_through`]).
///
/// It is not `#[non_exhaustive]`: a host matches on every way a currency
/// ends, so a way added later is a breaking release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expiry {
    /// On a date: which day it is depends on where the `Expiry` is from.
    On(Date),
    /// Never: once met, the requirement stays met. A requirement held until a
    /// later check replaces the one it rests on is this too, until that check
    /// gives a new result.
    Never,
}

/// Outcome of a single currency check.
///
/// `expiry` is computed whenever at least `required` qualifying events have
/// ever occurred, regardless of the look-back window, so a caller can report
/// both "expires on X" (future) and "expired on X" (past). It is `None` when
/// the pilot has never accumulated enough qualifying events, and for a check
/// requiring none, which is meaningless.
///
/// A check gives [`Expiry::Never`] only once the requirement is met by
/// `reference`. The counting fields (`found`, `qualifying`,
/// `qualifying_assessments`, `look_back_days`) are then as the check counted
/// them, which need not make up `required`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyResult {
    /// `On` the first day the pilot is not current, or `Never`.
    pub expiry: Option<Expiry>,
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
        match self.expiry {
            Some(Expiry::On(expiry)) => expiry > self.reference,
            Some(Expiry::Never) => true,
            None => false,
        }
    }

    /// The last day the pilot is (or was) current: `On` the day before
    /// `expiry`'s, which is the first day they are not, or `Never` when it is
    /// `Never`. `None` when they never have been, and for an `expiry` of
    /// `On(Date::MIN)`, which has no day before it.
    pub fn current_through(&self) -> Option<Expiry> {
        match self.expiry? {
            Expiry::On(expiry) => expiry.checked_sub(Duration::days(1)).map(Expiry::On),
            Expiry::Never => Some(Expiry::Never),
        }
    }

    /// Further qualifying events needed to become current: none when
    /// current, whatever `found` is.
    pub fn needed(&self) -> usize {
        if self.is_current() {
            0
        } else {
            self.required.saturating_sub(self.found)
        }
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

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    fn expiring(expiry: Option<Expiry>) -> CurrencyResult {
        CurrencyResult {
            expiry,
            reference: date!(2026 - 11 - 26),
            qualifying: Vec::new(),
            qualifying_assessments: Vec::new(),
            found: 0,
            required: 1,
            look_back_days: 0,
        }
    }

    #[test]
    fn a_result_that_never_expires_is_current_through_never() {
        let result = expiring(Some(Expiry::Never));
        assert!(result.is_current());
        assert_eq!(result.current_through(), Some(Expiry::Never));
    }

    #[test]
    fn a_result_that_never_expires_needs_nothing_with_none_found() {
        let result = expiring(Some(Expiry::Never));
        assert_eq!(result.found, 0);
        assert_eq!(result.needed(), 0);
        assert_eq!(expiring(None).needed(), 1);
    }

    #[test]
    fn a_result_is_current_through_the_day_before_it_expires() {
        let result = expiring(Some(Expiry::On(date!(2026 - 11 - 27))));
        assert!(result.is_current());
        assert_eq!(
            result.current_through(),
            Some(Expiry::On(date!(2026 - 11 - 26)))
        );
        assert!(!expiring(Some(Expiry::On(date!(2026 - 11 - 26)))).is_current());
    }

    #[test]
    fn a_result_never_met_is_not_current_through_any_day() {
        let result = expiring(None);
        assert!(!result.is_current());
        assert_eq!(result.current_through(), None);
    }
}
