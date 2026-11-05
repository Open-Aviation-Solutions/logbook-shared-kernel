// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Totals over a logbook's entries.
//!
//! Every total is the same accumulator, [`Total`], over a different
//! [`Measure`]: how much one entry contributes to that figure, which is zero
//! when it does not count towards it. Adding a figure means adding a measure,
//! and nothing that exists changes.

use std::marker::PhantomData;

use time::Date;

use super::accumulator::EntryAccumulator;
use crate::entry::Entry;

/// How much one entry contributes to a total; zero if it does not count.
pub trait Measure {
    fn measure(entry: &Entry) -> u64;
}

/// Sums a [`Measure`] over the entries inside an optional date range.
pub struct Total<M: Measure> {
    range: DateRange,
    total: u64,
    measure: PhantomData<M>,
}

impl<M: Measure> Total<M> {
    pub fn new(range: DateRange) -> Self {
        Self {
            range,
            total: 0,
            measure: PhantomData,
        }
    }

    pub fn total(&self) -> u64 {
        self.total
    }
}

// By hand, so that a measure need not be `Debug` itself.
impl<M: Measure> std::fmt::Debug for Total<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Total")
            .field("range", &self.range)
            .field("total", &self.total)
            .finish()
    }
}

impl<M: Measure> EntryAccumulator for Total<M> {
    fn accept_entry(&mut self, entry: &Entry) {
        if self.range.contains(entry.date()) {
            self.total += M::measure(entry);
        }
    }
}

/// An optional, inclusive date filter, held by each [`Total`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DateRange {
    pub from: Option<Date>,
    pub until: Option<Date>,
}

impl DateRange {
    pub fn contains(&self, date: Date) -> bool {
        self.from.is_none_or(|from| date >= from) && self.until.is_none_or(|until| date <= until)
    }
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn no_bounds_contains_every_date() {
        assert!(DateRange::default().contains(date!(1900 - 01 - 01)));
    }

    #[test]
    fn from_excludes_earlier_dates_and_includes_the_boundary() {
        let range = DateRange {
            from: Some(date!(2024 - 06 - 01)),
            until: None,
        };
        assert!(!range.contains(date!(2024 - 05 - 31)));
        assert!(range.contains(date!(2024 - 06 - 01)));
        assert!(range.contains(date!(2024 - 06 - 15)));
    }

    #[test]
    fn until_excludes_later_dates_and_includes_the_boundary() {
        let range = DateRange {
            from: None,
            until: Some(date!(2024 - 06 - 30)),
        };
        assert!(range.contains(date!(2024 - 06 - 30)));
        assert!(!range.contains(date!(2024 - 07 - 01)));
    }

    #[test]
    fn both_bounds_together_are_inclusive_on_each_side() {
        let range = DateRange {
            from: Some(date!(2024 - 01 - 01)),
            until: Some(date!(2024 - 06 - 30)),
        };
        assert!(!range.contains(date!(2023 - 12 - 31)));
        assert!(range.contains(date!(2024 - 03 - 01)));
        assert!(!range.contains(date!(2024 - 07 - 01)));
    }
}
