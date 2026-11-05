// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The bookkeeping that "N events in a look-back" checks share.

use time::{Date, Duration};

use super::currency::{Contribution, CurrencyResult};
use super::window::{add_months_clamped, count_in_window, months_on, nth_most_recent};
use crate::entry::{Assessment, AssessmentId, Entry, EntryId};

/// How far back a check looks: a number of days, or of calendar months.
#[derive(Debug, Clone, Copy)]
enum LookBack {
    Days(u32),
    Months(u32),
}

impl LookBack {
    /// The last day before the look-back from `reference` begins.
    fn cutoff(self, reference: Date) -> Date {
        match self {
            Self::Days(days) => reference.saturating_sub(Duration::days(days.into())),
            Self::Months(months) => i32::try_from(months)
                .ok()
                .and_then(|months| add_months_clamped(reference, -months))
                .unwrap_or(Date::MIN),
        }
    }

    /// The first day an event on `d` is outside the look-back.
    fn expiry(self, d: Date) -> Option<Date> {
        match self {
            Self::Days(days) => d.checked_add(Duration::days(days.into())),
            Self::Months(months) => i32::try_from(months)
                .ok()
                .and_then(|months| months_on(d, months)),
        }
    }
}

/// The bookkeeping every "N events in D days" (or "in M months") check shares:
/// every qualifying event date ever seen, and the entries or assessments that
/// fall inside the look-back window.
///
/// An authority writes a check as an accumulator that decides which events
/// of an entry or assessment qualify, records them here, and answers
/// [`EventWindow::result`].
#[derive(Debug, Clone)]
pub struct EventWindow {
    reference: Date,
    cutoff: Date,
    look_back: LookBack,
    required: usize,
    event_dates: Vec<Date>,
    qualifying: Vec<Contribution>,
    qualifying_assessments: Vec<AssessmentId>,
}

impl EventWindow {
    /// A window of `window_days` days.
    pub fn new(reference: Date, window_days: u32, required: usize) -> Self {
        Self::looking_back(reference, LookBack::Days(window_days), required)
    }

    /// A window of `months` calendar months: an event on 9 December is inside
    /// a 6-month window through 8 June (see [`months_on`]).
    pub fn months(reference: Date, months: u32, required: usize) -> Self {
        Self::looking_back(reference, LookBack::Months(months), required)
    }

    fn looking_back(reference: Date, look_back: LookBack, required: usize) -> Self {
        Self {
            reference,
            cutoff: look_back.cutoff(reference),
            look_back,
            required,
            event_dates: Vec::new(),
            qualifying: Vec::new(),
            qualifying_assessments: Vec::new(),
        }
    }

    /// Record `count` qualifying events from one entry, unless it is dated
    /// after `reference`: "as at" a date means what had happened by then, so
    /// a later-dated entry has not happened yet and does not count.
    pub fn record(&mut self, entry: &Entry, count: usize) {
        self.record_on(entry.id(), entry.date(), count);
    }

    /// [`Self::record`], for events an entry is reported by but is not the
    /// whole of (a flight logged in several entries).
    pub fn record_on(&mut self, entry: &EntryId, on: Date, count: usize) {
        if on > self.reference {
            return;
        }
        self.event_dates.extend(std::iter::repeat_n(on, count));
        if count > 0 && on > self.cutoff {
            self.qualifying.push(Contribution {
                entry: entry.clone(),
                events: count,
            });
        }
    }

    /// Record one qualifying assessment, unless it is dated after `reference`
    /// (see [`Self::record`]).
    pub fn record_assessment(&mut self, assessment: &Assessment) {
        if assessment.date > self.reference {
            return;
        }
        self.event_dates.push(assessment.date);
        if assessment.date > self.cutoff {
            self.qualifying_assessments.push(assessment.id.clone());
        }
    }

    /// The check's result, from what has been recorded.
    pub fn result(&self) -> CurrencyResult {
        let mut qualifying = self.qualifying.clone();
        qualifying.sort_by(|a, b| (&a.entry, a.events).cmp(&(&b.entry, b.events)));
        let mut qualifying_assessments = self.qualifying_assessments.clone();
        qualifying_assessments.sort();
        CurrencyResult {
            expiry_date: nth_most_recent(&self.event_dates, self.required)
                .and_then(|date| self.look_back.expiry(date)),
            reference: self.reference,
            qualifying,
            qualifying_assessments,
            found: count_in_window(&self.event_dates, self.cutoff),
            required: self.required,
            look_back_days: (self.reference - self.cutoff).whole_days(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use icao_shared_kernel::{Aircraft, Capacity, FlightDuration};
    use time::macros::date;
    use uuid::Uuid;

    use super::*;
    use crate::entry::{DayNight, EntryParts, InstrumentTime, Subject};

    const REFERENCE: Date = date!(2024 - 09 - 01);

    fn entry(id: &str, on: Date) -> Entry {
        Entry::new(EntryParts {
            id: EntryId::new(id),
            date: on,
            subject: Subject::Flight {
                flight: Uuid::nil(),
                aircraft: Aircraft::create("C172", "VH", "ABC").unwrap(),
                description: None,
                sole_occupant: false,
            },
            route: None,
            flight_rules: None,
            flight_time: FlightDuration::new(60).unwrap(),
            night: FlightDuration::new(0).unwrap(),
            capacity: Capacity::PilotInCommand,
            pilot_in_command: None,
            flight_crew: Vec::new(),
            instrument: InstrumentTime::default(),
            approaches: Vec::new(),
            take_offs: DayNight::default(),
            landings: DayNight::default(),
            sections: BTreeMap::new(),
        })
        .unwrap()
    }

    fn assessment(id: &str, on: Date) -> Assessment {
        Assessment {
            id: AssessmentId::new(id),
            date: on,
            flight: Uuid::nil(),
            sections: BTreeMap::new(),
        }
    }

    #[test]
    fn an_entry_dated_after_the_reference_does_not_count_at_all() {
        let mut window = EventWindow::new(REFERENCE, 90, 1);
        window.record(&entry("a", REFERENCE + Duration::days(1)), 1);
        let result = window.result();
        assert_eq!(result.found, 0);
        assert_eq!(result.qualifying, []);
        assert_eq!(result.expiry_date, None); // not just outside the window: never seen
    }

    #[test]
    fn an_entry_dated_on_the_reference_counts() {
        let mut window = EventWindow::new(REFERENCE, 90, 1);
        window.record(&entry("a", REFERENCE), 1);
        assert!(window.result().is_current());
    }

    #[test]
    fn an_assessment_dated_after_the_reference_does_not_count_at_all() {
        let mut window = EventWindow::new(REFERENCE, 90, 1);
        window.record_assessment(&assessment("a", REFERENCE + Duration::days(1)));
        let result = window.result();
        assert_eq!(result.found, 0);
        assert_eq!(result.qualifying_assessments, []);
        assert_eq!(result.expiry_date, None);
    }

    #[test]
    fn the_longest_look_backs_do_not_panic() {
        let mut days = EventWindow::new(REFERENCE, u32::MAX, 1);
        days.record(&entry("a", REFERENCE), 1);
        assert_eq!(days.result().found, 1);
        let mut months = EventWindow::months(REFERENCE, u32::MAX, 1);
        months.record(&entry("a", REFERENCE), 1);
        assert_eq!(months.result().found, 1);
    }

    #[test]
    fn the_qualifying_entries_do_not_depend_on_the_order_they_arrive_in() {
        let mut window = EventWindow::new(REFERENCE, 90, 1);
        window.record(&entry("b", REFERENCE), 1);
        window.record(&entry("a", REFERENCE), 2);
        let ids: Vec<_> = window
            .result()
            .qualifying
            .into_iter()
            .map(|contribution| contribution.entry)
            .collect();
        assert_eq!(ids, [EntryId::new("a"), EntryId::new("b")]);
    }
}
