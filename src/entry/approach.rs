// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An instrument approach the pilot conducted.

use std::fmt;

/// The type of instrument approach procedure, as recorded, in ICAO's terms.
/// Whether it is a 2D or a 3D approach operation (the classes of approach
/// operation in ICAO Annex 6) is derived by [`is_3d`](Self::is_3d); how an
/// authority counts it is that authority's reading.
///
/// A procedure particular to one State is recorded in that authority's
/// section, not here. There is no `Mls` value until one is needed.
///
/// Not `#[non_exhaustive]`, deliberately: an authority mapping approaches to
/// its rules should be made to handle each new value, not fall into a
/// wildcard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApproachType {
    /// A non-directional beacon approach: a 2D approach operation.
    Ndb,
    /// A VOR approach: a 2D approach operation.
    Vor,
    /// A localizer approach, without the glide path of an instrument landing
    /// system: a 2D approach operation.
    Localizer,
    /// An instrument landing system approach (Annex 10, Volume I), with
    /// vertical guidance from the glide path: a 3D approach operation.
    Ils,
    /// A GBAS landing system approach (Annex 10, Volume I): a 3D approach
    /// operation.
    Gls,
    /// An RNP APCH approach (Doc 9613, the Performance-based Navigation
    /// Manual), flown to the given minima.
    RnpApch { minima: RnpApchMinima },
}

/// The minima an RNP APCH approach is flown to (Doc 9613, the
/// Performance-based Navigation Manual).
///
/// Not `#[non_exhaustive]`, deliberately, for the reason `ApproachType` is
/// not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RnpApchMinima {
    /// LNAV minima: lateral guidance only, a 2D approach operation.
    Lnav,
    /// LNAV/VNAV minima: lateral and vertical guidance, a 3D approach
    /// operation.
    LnavVnav,
    /// LP minima ("localizer performance"): a 2D
    /// approach operation.
    Lp,
    /// LPV minima ("localizer performance with vertical guidance"): a 3D
    /// approach operation.
    Lpv,
}

impl ApproachType {
    /// Whether this is a 3D approach operation (lateral and vertical
    /// guidance, Annex 6), as against a 2D one (lateral guidance only).
    pub fn is_3d(self) -> bool {
        match self {
            Self::Ndb | Self::Vor | Self::Localizer => false,
            Self::Ils | Self::Gls => true,
            Self::RnpApch { minima } => minima.is_3d(),
        }
    }
}

impl RnpApchMinima {
    /// Whether these minima make the approach a 3D approach operation.
    pub fn is_3d(self) -> bool {
        matches!(self, Self::LnavVnav | Self::Lpv)
    }
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
