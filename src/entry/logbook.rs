// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! What an authority reads to work out its currency.

use std::collections::BTreeMap;

use time::Date;
use uuid::Uuid;

use super::{AuthorityId, AuthoritySpecificInformation, Entry};

/// Identifies an assessment. Opaque, as [`EntryId`](super::EntryId) is.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssessmentId(String);

impl AssessmentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An assessment of the pilot (a flight test, review or check) conducted on
/// a flight. What kind it was, its outcome and its purpose are each
/// authority's, in its authority-specific information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    pub id: AssessmentId,
    /// The date of the flight it was conducted on.
    pub date: Date,
    /// The kernel's `Flight` it was conducted on.
    pub flight: Uuid,
    pub authority_information: BTreeMap<AuthorityId, AuthoritySpecificInformation>,
}

/// A pilot's entries and assessments.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Logbook {
    pub entries: Vec<Entry>,
    pub assessments: Vec<Assessment>,
}
