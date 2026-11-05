// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An authority's section of an entry.

/// What one authority adds to or narrows in the common part of an entry,
/// stored as the authority gives it. Only that authority reads inside it;
/// this crate and the host hand it back unread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section(serde_json::Value);

impl Section {
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &serde_json::Value {
        &self.0
    }

    pub fn into_value(self) -> serde_json::Value {
        self.0
    }
}
