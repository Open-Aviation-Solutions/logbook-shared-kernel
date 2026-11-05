// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod common;

use common::flight;
use icao_shared_kernel::Capacity;
use logbook_shared_kernel::{DateRange, Entry, EntryAccumulator, Measure, Total};
use time::macros::date;

struct FlightTime;

impl Measure for FlightTime {
    fn measure(entry: &Entry) -> u64 {
        entry.flight_time().total_minutes().unsigned_abs()
    }
}

#[test]
fn a_total_counts_only_the_entries_dated_in_its_range() {
    // The entry is dated 5 November 2026.
    let entry = flight("e", Capacity::PilotInCommand);
    let total = |from, until| {
        let mut total = Total::<FlightTime>::new(DateRange { from, until });
        total.accept_entry(&entry);
        total.total()
    };
    let on = Some(date!(2026 - 11 - 05));
    assert_eq!(total(on, on), 60);
    assert_eq!(total(Some(date!(2026 - 11 - 06)), None), 0);
    assert_eq!(total(None, Some(date!(2026 - 11 - 04))), 0);
}
