// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An instrument approach the pilot conducted.

use icao_shared_kernel::{ApproachType, LocationIndicator};

/// An instrument approach the pilot conducted, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approach {
    procedure: ApproachType,
    aerodrome: LocationIndicator,
}

impl Approach {
    /// An approach at `aerodrome`, given by its ICAO location indicator
    /// (Doc 7910).
    pub fn new(procedure: ApproachType, aerodrome: LocationIndicator) -> Self {
        Self {
            procedure,
            aerodrome,
        }
    }

    pub fn procedure(&self) -> ApproachType {
        self.procedure
    }

    pub fn aerodrome(&self) -> &LocationIndicator {
        &self.aerodrome
    }
}
