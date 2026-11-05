// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An instrument approach the pilot conducted.

use std::fmt;

/// The type of instrument approach procedure (CASR 1998 reg 61.345(3)(k)),
/// as recorded. It says what was flown, not what it counts as: whether a type
/// is a 2D or a 3D approach is each authority's reading.
///
/// The values are pilot-logbook's, moved as they are. No ICAO text that lists
/// types of approach procedure is quoted in these repositories; each value's
/// words are from CASR 1998 Table 61.935, Part 4 (the approach endorsements),
/// item by item, where the table has them. They do not yet line up with the
/// table (`Rnp` covers both items 21 and 22, the 2D and the 3D operation):
/// that is logbook-shared-kernel#6.
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
    /// An approach at `aerodrome`, given by its ICAO location indicator
    /// (Doc 7910): 4 capital letters.
    pub fn new(
        procedure: ApproachType,
        aerodrome: impl Into<String>,
    ) -> Result<Self, ApproachError> {
        let aerodrome = aerodrome.into();
        if aerodrome.len() != 4 || !aerodrome.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(ApproachError::Aerodrome(aerodrome));
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

/// Why [`Approach::new`] refused an approach. Its `Display` is plain English,
/// for logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApproachError {
    /// An aerodrome that is not an ICAO location indicator: 4 capital
    /// letters.
    Aerodrome(String),
}

impl fmt::Display for ApproachError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aerodrome(aerodrome) => write!(
                f,
                "aerodrome '{aerodrome}' must be an ICAO location indicator: 4 capital letters"
            ),
        }
    }
}

impl std::error::Error for ApproachError {}
