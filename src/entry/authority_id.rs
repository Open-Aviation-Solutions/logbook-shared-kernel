// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The identity of a licensing authority.

use std::fmt;

use icao_shared_kernel::{IssuingState, Licence, ValidationError};

/// A licensing authority: the State and the authority's code together, as a
/// licence's `issuing_state` and `issuing_authority` are, written
/// `AU-CASA`. The code alone is not unique across States: the United
/// Kingdom and New Zealand each have a "CAA". It keys an entry's sections.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthorityId {
    state: IssuingState,
    authority: String,
}

impl AuthorityId {
    /// Refuses a State that [`IssuingState::parse`] refuses, and an empty
    /// authority.
    pub fn new(
        state: impl Into<String>,
        authority: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let state = IssuingState::parse(state)?;
        let authority = authority.into();
        if authority.is_empty() {
            return Err(ValidationError::Empty {
                field: "issuing_authority",
            });
        }
        Ok(Self { state, authority })
    }

    /// The authority that issued a licence.
    pub fn of(licence: &Licence) -> Self {
        Self {
            state: licence.issuing_state().clone(),
            authority: licence.issuing_authority().to_owned(),
        }
    }

    pub fn state(&self) -> &str {
        self.state.as_str()
    }

    pub fn authority(&self) -> &str {
        &self.authority
    }
}

// Ordered by the key's spelling, as `AU-CASA`'s parts: `IssuingState` is not
// `Ord`.
impl PartialOrd for AuthorityId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AuthorityId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.state.as_str(), &self.authority).cmp(&(other.state.as_str(), &other.authority))
    }
}

impl fmt::Display for AuthorityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.state.as_str(), self.authority)
    }
}
