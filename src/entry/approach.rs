// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An instrument approach the pilot conducted.

use icao_shared_kernel::{ApproachType, LocationIndicator};

/// An instrument approach the pilot conducted, and where, and whether it
/// ended in a circling manoeuvre.
///
/// `procedure` is the procedure flown, and `procedure().is_3d()` whether it was
/// a 2D or a 3D operation (Annex 6). `circled` is whether it was extended by
/// a circling approach, which does not change that class: Annex 6 Part I
/// names a "circling approach operation" beside a 2D one, in its definition
/// of minimum descent altitude or height, and does not class a circled
/// approach as 2D. An authority reads `circled()` for its own rules on
/// circling approaches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approach {
    procedure: ApproachType,
    aerodrome: LocationIndicator,
    circled: bool,
}

impl Approach {
    /// An approach at `aerodrome`, given by its ICAO location indicator
    /// (Doc 7910). `circled` is whether the approach ended in a circling
    /// manoeuvre.
    pub fn new(procedure: ApproachType, aerodrome: LocationIndicator, circled: bool) -> Self {
        Self {
            procedure,
            aerodrome,
            circled,
        }
    }

    pub fn procedure(&self) -> ApproachType {
        self.procedure
    }

    pub fn aerodrome(&self) -> &LocationIndicator {
        &self.aerodrome
    }

    /// Whether the approach ended in a circling manoeuvre.
    pub fn circled(&self) -> bool {
        self.circled
    }
}
