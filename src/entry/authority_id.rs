// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The identity of a licensing authority.

use std::fmt;

use icao_shared_kernel::{Licence, ValidationError};

/// A licensing authority: the State and the authority's code together, as a
/// licence's `issuing_state` and `issuing_authority` are, written
/// `AU-CASA`. The code alone is not unique across States: the United
/// Kingdom and New Zealand each have a "CAA". It keys an entry's sections.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AuthorityId {
    state: String,
    authority: String,
}

impl AuthorityId {
    /// Refuses what [`Licence::new`] refuses of these two parts: a State
    /// that is not two capital letters (an ISO 3166-1 alpha-2 code, by
    /// shape), and an empty authority.
    pub fn new(
        state: impl Into<String>,
        authority: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let state = state.into();
        let authority = authority.into();
        if state.len() != 2 || !state.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(ValidationError::IssuingState(state));
        }
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
            state: licence.issuing_state().to_string(),
            authority: licence.issuing_authority().to_owned(),
        }
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn authority(&self) -> &str {
        &self.authority
    }
}

impl fmt::Display for AuthorityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.state, self.authority)
    }
}
