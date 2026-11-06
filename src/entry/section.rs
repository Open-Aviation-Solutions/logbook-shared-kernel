// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An authority's section of an entry.

/// What one authority adds to or narrows in the common part of an entry,
/// stored as the authority gives it. It is the authority's: the host stores
/// in it the answers to the authority's form, each under its question's key
/// (a JSON object, in the shapes [`Answer`](crate::authority::Answer)
/// describes), and [`Form`](crate::authority::Form) reads those answers to
/// decide which questions are shown. Anything else in it only the authority
/// reads.
///
/// It is a stored format, so it carries the version of the form its answers
/// were given to, under the reserved key [`Section::VERSION_KEY`], which the
/// host writes with [`Form::stamp`](crate::authority::Form::stamp) and the
/// authority reads with [`Section::version`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section(serde_json::Value);

impl Section {
    /// The key the section's version is stored under, as a JSON number. It
    /// is reserved: no question may use it.
    pub const VERSION_KEY: &'static str = "_version";

    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }

    /// The version of the form whose answers the section holds, for the
    /// authority to read an older one or refuse it. `None`: the key is
    /// absent, or holds no version (not a whole number from 0 to
    /// `u32::MAX`), or the section is not a JSON object.
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
