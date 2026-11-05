// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Where a flight went.

use icao_shared_kernel::{SignificantPoint, Waypoint};

/// Where a flight began and ended, and the points it passed on the way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub departure: SignificantPoint,
    pub arrival: SignificantPoint,
    /// The en-route points, in order, not counting the departure and the
    /// arrival.
    pub waypoints: Vec<Waypoint>,
}
