// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! What an authority's questions and information are about.

use super::{Assessment, AuthorityId, AuthoritySpecificInformation, Entry};

/// A record in the logbook that holds authority-specific information: what
/// that information, and the authority's questions for it, are about. The
/// host says which at each call, and the authority answers for each kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Record<'a> {
    Entry(&'a Entry),
    Assessment(&'a Assessment),
}

impl<'a> Record<'a> {
    /// `authority`'s information on the record, if it has any.
    pub fn information_for(
        &self,
        authority: &AuthorityId,
    ) -> Option<&'a AuthoritySpecificInformation> {
        match self {
            Self::Entry(entry) => entry.authority_information().get(authority),
            Self::Assessment(assessment) => assessment.authority_information.get(authority),
        }
    }
}
