// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! What an authority's rules are written over: the accumulators that fold
//! a logbook into a result in one pass, the currency checks and their
//! event window, and the totals.

mod accumulator;
mod currency;
mod event_window;
mod totals;
pub mod window;

pub use accumulator::{run_assessments, AssessmentAccumulator, EntryAccumulator};
pub use currency::{AssessmentCurrencyCheck, Contribution, CurrencyCheck, CurrencyResult};
pub use event_window::EventWindow;
pub use totals::{DateRange, Measure, Total};
