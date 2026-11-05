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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section(serde_json::Value);

impl Section {
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
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
