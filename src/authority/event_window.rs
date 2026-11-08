// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The bookkeeping that "N events in a look-back" checks share.

use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

use time::{Date, Duration};

use super::currency::{Contribution, CurrencyResult};
use super::window::{add_months_clamped, months_on};
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
/// how many qualifying events there were on each date ever seen, and the
/// entries or assessments that fall inside the look-back window.
///
/// It keeps a count for each date, not a date for each event, so a count
/// from a pilot's answer, however large, is never expanded.
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
    events_on: BTreeMap<Date, usize>,
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
            events_on: BTreeMap::new(),
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
        if count == 0 {
            return;
        }
        self.add_events(on, count);
        if on > self.cutoff {
            // An entry recorded more than once is one contribution, with
            // all its events.
            match self
                .qualifying
                .iter_mut()
                .find(|contribution| contribution.entry == *entry)
            {
                Some(contribution) => {
                    contribution.events = contribution.events.saturating_add(count);
                }
                None => self.qualifying.push(Contribution {
                    entry: entry.clone(),
                    events: count,
                }),
            }
        }
    }

    /// Record one qualifying assessment, unless it is dated after `reference`
    /// (see [`Self::record`]).
    pub fn record_assessment(&mut self, assessment: &Assessment) {
        if assessment.date > self.reference {
            return;
        }
        self.add_events(assessment.date, 1);
        if assessment.date > self.cutoff {
            self.qualifying_assessments.push(assessment.id.clone());
        }
    }

    fn add_events(&mut self, on: Date, count: usize) {
        let events = self.events_on.entry(on).or_insert(0);
        *events = events.saturating_add(count);
    }

    /// The date of the `required`-th most recent event, which the check
    /// expires from. It is taken over every event, not only those in the
    /// look-back, so a result can say both "expires on X" and "expired on X".
    /// `None` when there are fewer events than that, or none are required.
    fn date_of_required_event(&self) -> Option<Date> {
        if self.required == 0 {
            return None;
        }
        let mut seen: usize = 0;
        self.events_on.iter().rev().find_map(|(on, events)| {
            seen = seen.saturating_add(*events);
            (seen >= self.required).then_some(*on)
        })
    }

    /// The check's result, from what has been recorded.
    pub fn result(&self) -> CurrencyResult {
        let mut qualifying = self.qualifying.clone();
        qualifying.sort_by(|a, b| a.entry.cmp(&b.entry));
        let mut qualifying_assessments = self.qualifying_assessments.clone();
        qualifying_assessments.sort();
        CurrencyResult {
            expiry_date: self
                .date_of_required_event()
                .and_then(|date| self.look_back.expiry(date)),
            reference: self.reference,
            qualifying,
            qualifying_assessments,
            found: self
                .events_on
                .range((Excluded(self.cutoff), Unbounded))
                .fold(0, |found: usize, (_, events)| found.saturating_add(*events)),
            required: self.required,
            look_back_days: (self.reference - self.cutoff).whole_days(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use icao_shared_kernel::{
        Aircraft, AircraftRegistration, AircraftType, Capacity, FlightDuration, NationalityMark,
    };
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
                aircraft: Aircraft::new(
                    AircraftType::parse("C172").unwrap(),
                    AircraftRegistration::new(NationalityMark::parse("VH").unwrap(), "ABC")
                        .unwrap(),
                ),
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
            authority_information: BTreeMap::new(),
        })
        .unwrap()
    }

    fn assessment(id: &str, on: Date) -> Assessment {
        Assessment {
            id: AssessmentId::new(id),
            date: on,
            flight: Uuid::nil(),
            authority_information: BTreeMap::new(),
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
    fn an_entry_recorded_twice_is_one_contribution_with_all_its_events() {
        let mut window = EventWindow::new(REFERENCE, 90, 3);
        let a = entry("a", REFERENCE);
        window.record(&a, 1);
        window.record(&a, 2);
        let result = window.result();
        assert_eq!(
            result.qualifying,
            [Contribution {
                entry: EntryId::new("a"),
                events: 3
            }]
        );
        assert!(result.is_current());
    }

    #[test]
    fn a_large_count_is_reported_as_recorded() {
        let mut window = EventWindow::new(REFERENCE, 90, 3);
        window.record(&entry("a", REFERENCE), 4_000_000_000);
        let result = window.result();
        assert_eq!(result.found, 4_000_000_000);
        assert_eq!(
            result.qualifying,
            [Contribution {
                entry: EntryId::new("a"),
                events: 4_000_000_000
            }]
        );
        assert_eq!(result.expiry_date, Some(REFERENCE + Duration::days(90)));
    }

    #[test]
    fn the_check_expires_from_the_date_of_the_event_that_makes_up_the_number_required() {
        let mut window = EventWindow::new(REFERENCE, 90, 3);
        window.record(&entry("a", REFERENCE - Duration::days(10)), 2);
        window.record(&entry("b", REFERENCE - Duration::days(20)), 2);
        window.record(&entry("c", REFERENCE - Duration::days(100)), 5);
        let result = window.result();
        assert_eq!(result.found, 4);
        assert_eq!(
            result.expiry_date,
            Some(REFERENCE - Duration::days(20) + Duration::days(90))
        );
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
