// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An instrument approach the pilot conducted.

use super::EntryError;

/// The type of instrument approach procedure (CASR 1998 reg 61.345(3)(k)),
/// as recorded. It says what was flown, not what it counts as: whether a type
/// is a 2D or a 3D approach is each authority's reading.
///
/// The values are pilot-logbook's, moved as they are. No ICAO text that lists
/// types of approach procedure is quoted in these repositories; each value's
/// words are from CASR 1998 Table 61.935, Part 4 (the approach endorsements),
/// item by item, where the table has them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApproachType {
    /// Using "an instrument landing system" (item 23).
    Ils,
    /// A "required navigational performance instrument approach operation"
    /// (items 21 and 22).
    Rnp,
    /// Area navigation: "another kind of area navigation-based system"
    /// (item 21).
    Rnav,
    /// Using "a global navigation satellite system" (items 20 and 21).
    Gnss,
    /// Using "a VHF omni-range/localiser navigation system" (item 19).
    Vor,
    /// Using "a non-directional beacon navigation system" (item 18).
    Ndb,
    /// A localizer approach. The table names the localiser only with the
    /// VHF omni-range, in "VOR/LLZ" (item 19).
    Localizer,
}

/// An instrument approach the pilot conducted, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approach {
    procedure: ApproachType,
    aerodrome: String,
}

impl Approach {
    /// An approach at `aerodrome`, its location indicator: 3 or 4 capital
    /// letters.
    pub fn new(procedure: ApproachType, aerodrome: impl Into<String>) -> Result<Self, EntryError> {
        let aerodrome = aerodrome.into();
        let length = aerodrome.chars().count();
        if !(3..=4).contains(&length) || !aerodrome.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(EntryError::Aerodrome(aerodrome));
        }
        Ok(Self {
            procedure,
            aerodrome,
        })
    }

    pub fn procedure(&self) -> ApproachType {
        self.procedure
    }

    pub fn aerodrome(&self) -> &str {
        &self.aerodrome
    }
}
