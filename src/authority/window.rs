// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The date arithmetic that currency checks share.

use time::{Date, Month};

/// The `n`-th most recent of `dates` (the first is the most recent), which a
/// check needing `n` events expires from. `None` when there are fewer than
/// `n`, and for an `n` of zero, which is meaningless. A check takes it over
/// every event, not only those in its look-back window, so it can report both
/// "expires on X" and "expired on X".
pub fn nth_most_recent(dates: &[Date], n: usize) -> Option<Date> {
    let index = n.checked_sub(1)?;
    let mut sorted = dates.to_vec();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    sorted.get(index).copied()
}

/// The first day on which `d` is no longer within the previous `months`
/// calendar months, as [`add_months_clamped`] looks back: the same day of the
/// month, `months` on, or where that month is too short for it, the first
/// day of the month after.
///
/// So a flight on 9 December is within the previous 6 months through 8 June,
/// and one on 31 August through the last day of February.
pub fn months_on(d: Date, months: i32) -> Option<Date> {
    let same_day = add_months_clamped(d, months)?;
    if same_day.day() == d.day() {
        Some(same_day)
    } else {
        same_day.next_day()
    }
}

/// How many events fall strictly after the cutoff.
pub fn count_in_window(all_event_dates: &[Date], cutoff: Date) -> usize {
    all_event_dates.iter().filter(|d| **d > cutoff).count()
}

/// Last day of the Nth calendar month after the month containing `d`.
///
/// The "end of the Nth month after the month in which…" a regulation can set
/// a currency by (CASA's reg 61.880(3)(a), for one). `None` only if the
/// result is outside the representable date range.
pub fn end_of_nth_month_after(d: Date, n: u32) -> Option<Date> {
    let total = u32::from(u8::from(d.month())) + n;
    let year = d.year() + i32::try_from((total - 1) / 12).ok()?;
    let month = Month::try_from(u8::try_from((total - 1) % 12 + 1).ok()?).ok()?;
    Date::from_calendar_date(year, month, month.length(year)).ok()
}

/// The date `months` calendar months from `d` (negative for earlier), keeping
/// the day of the month or, where that month is shorter, its last day.
///
/// `None` only if the result is outside the representable date range.
pub fn add_months_clamped(d: Date, months: i32) -> Option<Date> {
    let total = d.year() * 12 + i32::from(u8::from(d.month())) - 1 + months;
    let year = total.div_euclid(12);
    let month = Month::try_from(u8::try_from(total.rem_euclid(12) + 1).ok()?).ok()?;
    Date::from_calendar_date(year, month, d.day().min(month.length(year))).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn add_months_clamps_the_day_to_the_shorter_month() {
        let cases = [
            (date!(2026 - 08 - 31), 6, date!(2027 - 02 - 28)),
            (date!(2023 - 08 - 31), 6, date!(2024 - 02 - 29)),
            (date!(2026 - 05 - 31), -3, date!(2026 - 02 - 28)),
            (date!(2026 - 01 - 15), -3, date!(2025 - 10 - 15)),
            (date!(2025 - 11 - 20), 6, date!(2026 - 05 - 20)),
            (date!(2026 - 03 - 10), 0, date!(2026 - 03 - 10)),
            (date!(2026 - 03 - 10), -12, date!(2025 - 03 - 10)),
        ];
        for (from, months, expected) in cases {
            assert_eq!(
                add_months_clamped(from, months),
                Some(expected),
                "{from} {months}"
            );
        }
    }

    #[test]
    fn months_on_is_the_first_day_a_look_back_of_that_many_months_no_longer_reaches() {
        assert_eq!(
            months_on(date!(2025 - 12 - 09), 6),
            Some(date!(2026 - 06 - 09))
        );
        assert_eq!(
            months_on(date!(2025 - 08 - 31), 6),
            Some(date!(2026 - 03 - 01))
        );
        assert_eq!(
            months_on(date!(2023 - 08 - 31), 6),
            Some(date!(2024 - 03 - 01))
        );
        assert_eq!(
            months_on(date!(2023 - 08 - 29), 6),
            Some(date!(2024 - 02 - 29))
        );

        // Against the look-back itself: `d` is within the previous 6 months of
        // a day exactly when that day is before `months_on(d, 6)`.
        let mut d = date!(2023 - 01 - 01);
        while d < date!(2025 - 01 - 01) {
            let expiry = months_on(d, 6).unwrap();
            for reference in [expiry.previous_day().unwrap(), expiry] {
                let within = d > add_months_clamped(reference, -6).unwrap();
                assert_eq!(within, reference < expiry, "{d} on {reference}");
            }
            d = d.next_day().unwrap();
        }
    }

    #[test]
    fn end_of_nth_month_after_is_the_last_day_of_that_month() {
        assert_eq!(
            end_of_nth_month_after(date!(2025 - 03 - 15), 12),
            Some(date!(2026 - 03 - 31))
        );
        assert_eq!(
            end_of_nth_month_after(date!(2025 - 01 - 31), 12),
            Some(date!(2026 - 01 - 31))
        );
    }

    #[test]
    fn end_of_nth_month_after_handles_leap_years_and_year_rollover() {
        // Feb 2024 is a leap February; Feb 2025 is not.
        assert_eq!(
            end_of_nth_month_after(date!(2023 - 02 - 10), 12),
            Some(date!(2024 - 02 - 29))
        );
        assert_eq!(
            end_of_nth_month_after(date!(2024 - 02 - 10), 12),
            Some(date!(2025 - 02 - 28))
        );
        // Crossing a year boundary without landing on a multiple of 12.
        assert_eq!(
            end_of_nth_month_after(date!(2025 - 11 - 20), 3),
            Some(date!(2026 - 02 - 28))
        );
        assert_eq!(
            end_of_nth_month_after(date!(2025 - 12 - 31), 1),
            Some(date!(2026 - 01 - 31))
        );
    }

    #[test]
    fn nth_most_recent_counts_back_from_the_latest() {
        let dates = [
            date!(2026 - 01 - 01),
            date!(2026 - 03 - 01),
            date!(2026 - 02 - 01),
        ];
        assert_eq!(nth_most_recent(&dates, 1), Some(date!(2026 - 03 - 01)));
        assert_eq!(nth_most_recent(&dates, 3), Some(date!(2026 - 01 - 01)));
        assert_eq!(nth_most_recent(&dates, 4), None);
        assert_eq!(nth_most_recent(&dates, 0), None);
    }

    #[test]
    fn events_on_the_cutoff_are_outside_the_window() {
        let cutoff = date!(2026 - 03 - 01);
        let dates = [date!(2026 - 03 - 01), date!(2026 - 03 - 02)];
        assert_eq!(count_in_window(&dates, cutoff), 1);
    }
}
