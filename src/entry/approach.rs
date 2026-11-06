// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An instrument approach the pilot conducted.

use icao_shared_kernel::{ApproachType, LocationIndicator};

/// An instrument approach the pilot conducted, and where, and whether it
/// ended in a circling manoeuvre.
///
/// `procedure` is the procedure flown; it does not say what class of operation
/// the approach was. ICAO Annex 6 classes the operation flown, so an ILS
/// approach that ends in a circling manoeuvre is a 2D operation. This
/// crate derives no class of its own: an authority reads it from
/// `procedure().is_3d()` and `circled()`.
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
