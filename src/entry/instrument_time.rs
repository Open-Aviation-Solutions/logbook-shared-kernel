// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An entry's instrument time.

use icao_shared_kernel::FlightDuration;

/// Annex 1's instrument flight time, with the time in actual and in
/// simulated conditions kept apart, and its instrument ground time.
///
/// A coarser view sums the two flight parts (CASA's one figure, CASR 1998
/// reg 61.345(3)(j)); a finer one (New Zealand's "the simulated and actual
/// instrument flight time", rule 61.29(b)(2)(iii); 14 CFR 61.51(b)(3)) cannot
/// be made from a sum. On a flight only the flight parts can be logged, and
/// on a session only the ground time ([`Entry::new`](crate::entry::Entry::new)
/// refuses the rest).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstrumentTime {
    /// Instrument flight time in actual conditions, in the aircraft.
    pub actual: FlightDuration,
    /// Instrument flight time in simulated conditions, in the aircraft: in
    /// flight, with the outside view blocked. This is not ICAO's "simulated
    /// instrument flight" in a device, which is `ground`.
    pub simulated: FlightDuration,
    /// Instrument ground time: "practising, on the ground, simulated
    /// instrument flight in a flight simulation training device approved by
    /// the Licensing Authority" (Annex 1).
    pub ground: FlightDuration,
}

/// None of any kind.
impl Default for InstrumentTime {
    fn default() -> Self {
        let none = FlightDuration::new(0).expect("zero is not negative");
        Self {
            actual: none,
            simulated: none,
            ground: none,
        }
    }
}

impl InstrumentTime {
    /// Instrument flight time: actual and simulated together.
    pub fn flight(&self) -> FlightDuration {
        let minutes = self
            .actual
            .total_minutes()
            .saturating_add(self.simulated.total_minutes());
        FlightDuration::new(minutes)
            .expect("a sum of durations that are not negative is not negative")
    }
}
