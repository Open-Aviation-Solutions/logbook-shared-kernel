// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An instrument approach the pilot conducted.

use std::fmt;

use icao_shared_kernel::ApproachType;

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
/// for logs. Non-exhaustive: a refusal added later is not a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
