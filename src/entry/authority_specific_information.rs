// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An authority's own information on a record, an entry or an assessment.

/// What one authority adds to a record (an entry or an assessment), or
/// narrows in an entry's common part, stored as the authority gives it. It
/// is the authority's: the host stores in it the answers to the authority's
/// questions, each under its question's key (a JSON object, in the shapes
/// [`Answer`](crate::authority::Answer) describes), and
/// [`Questions`](crate::authority::Questions) reads those answers to decide
/// which questions apply. Anything else in it only the authority reads.
///
/// It is a stored format, so it carries the version of the questions its
/// answers were given to, under the reserved key
/// [`AuthoritySpecificInformation::VERSION_KEY`], which the host writes with
/// [`Questions::stamp`](crate::authority::Questions::stamp) and the authority
/// reads with [`AuthoritySpecificInformation::version`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoritySpecificInformation(serde_json::Value);

impl AuthoritySpecificInformation {
    /// The key the version is stored under, as a JSON number. It is
    /// reserved: no question may use it.
    pub const VERSION_KEY: &'static str = "_version";

    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }

    /// The version of the questions whose answers the information holds,
    /// for the authority to read an older one or refuse it. `None`: the key
    /// is absent, or holds no version (not a whole number from 0 to
    /// `u32::MAX`), or the information is not a JSON object.
    pub fn version(&self) -> Option<u32> {
        self.0
            .get(Self::VERSION_KEY)
            .and_then(serde_json::Value::as_u64)
            .and_then(|version| u32::try_from(version).ok())
    }

    pub fn value(&self) -> &serde_json::Value {
        &self.0
    }

    pub(crate) fn value_mut(&mut self) -> &mut serde_json::Value {
        &mut self.0
    }

    pub fn into_value(self) -> serde_json::Value {
        self.0
    }
}
