// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A count kept by day and by night.

/// A count kept by day and by night, as reg 61.345(3)(i) of CASR 1998 asks
/// "whether the flight was by day or night, or both". No regulation defines
/// "day": it is what is not night.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DayNight<T> {
    pub day: T,
    pub night: T,
}

impl<T: Copy + Into<u64>> DayNight<T> {
    /// Day and night together, held at `u64::MAX` should the two be more.
    pub fn total(&self) -> u64 {
        self.day.into().saturating_add(self.night.into())
    }
}
